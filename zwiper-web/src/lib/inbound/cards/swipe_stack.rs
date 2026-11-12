//! The swipe surface for a browser: the app's stack of cards, dragged with
//! the mouse or a finger, plus arrow keys and one button per direction.
//!
//! A drag commits past [`COMMIT_PX`] along its axis; a shorter one springs
//! back. A committed card flies off on the app's exit keyframes while the
//! host's handler runs at once, so the next card takes a drag right away. A
//! down swipe never moves the card: it is the host's undo. The keys and the
//! dock's buttons ([`SwipeDock`]) go through [`SwipeControl`], so they
//! animate the same way a drag does, and a press before the last card is half way out is dropped, so
//! two exits overlap at most by half.

use std::collections::HashSet;

use chrono::{DateTime, Utc};
use components::icons::{ARROW_DOWN, ARROW_LEFT, ARROW_RIGHT, ARROW_UP, DockIcon};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::{ImageSize, ScryfallData};
use zwipe_ui::stack::{Direction, STACK_DEPTH, ghost_held};
use zwipe_ui::telemetry::SwipeDirection;
use zwipe_ui::timing::TOP_PAINT_GRACE;

use crate::inbound::chrome::shortcuts::{self, Shortcut};
use crate::outbound::timer;

/// A drag this far along its axis, in pixels, commits; the app's distance.
pub const COMMIT_PX: f64 = 90.0;

/// Degrees of tilt per pixel of horizontal drag, as in the app.
const TILT_PER_PX: f64 = 0.06;

/// How long a card's exit runs on the web, in milliseconds: `web.css`'s
/// `--web-swipe-exit`.
const EXIT_MS: i64 = 320;

/// How long after a card starts flying off a key or button press plays:
/// half of [`EXIT_MS`], when the card is half way out.
const PRESS_GAP_MS: i64 = EXIT_MS / 2;

/// A direction a card can be swiped, down being the host's undo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Swipe {
    /// Left.
    Left,
    /// Right.
    Right,
    /// Up.
    Up,
    /// Down, which never moves the card.
    Down,
}

impl From<Swipe> for SwipeDirection {
    fn from(swipe: Swipe) -> Self {
        match swipe {
            Swipe::Left => Self::Left,
            Swipe::Right => Self::Right,
            Swipe::Up => Self::Up,
            Swipe::Down => Self::Down,
        }
    }
}

impl Swipe {
    /// The arrow key that swipes this way.
    pub fn key(self) -> &'static str {
        match self {
            Self::Left => "ArrowLeft",
            Self::Right => "ArrowRight",
            Self::Up => "ArrowUp",
            Self::Down => "ArrowDown",
        }
    }

    /// The arrow icon its dock button shows, a path for [`DockIcon`].
    pub fn arrow(self) -> &'static str {
        match self {
            Self::Left => ARROW_LEFT,
            Self::Right => ARROW_RIGHT,
            Self::Up => ARROW_UP,
            Self::Down => ARROW_DOWN,
        }
    }

    /// The direction a card leaves by, for the exit keyframe; `None` for down.
    fn exit(self) -> Option<Direction> {
        match self {
            Self::Left => Some(Direction::Left),
            Self::Right => Some(Direction::Right),
            Self::Up => Some(Direction::Up),
            Self::Down => None,
        }
    }

    /// Whether a key or button press of this swipe plays now, given how
    /// long ago, in milliseconds, the last card started flying off (`None`
    /// when none has). It plays once that card is half way out, so two exits
    /// overlap. Down always plays, having no exit.
    fn plays_after(self, since_last_exit_ms: Option<i64>) -> bool {
        self == Self::Down || since_last_exit_ms.is_none_or(|ms| ms >= PRESS_GAP_MS)
    }

    /// The swipe a drag of `(dx, dy)` pixels makes, if it went far enough.
    /// The longer axis decides.
    pub fn of_drag(dx: f64, dy: f64) -> Option<Self> {
        if dx.abs() >= dy.abs() {
            if dx >= COMMIT_PX {
                Some(Self::Right)
            } else if dx <= -COMMIT_PX {
                Some(Self::Left)
            } else {
                None
            }
        } else if dy <= -COMMIT_PX {
            Some(Self::Up)
        } else if dy >= COMMIT_PX {
            Some(Self::Down)
        } else {
            None
        }
    }
}

/// A swipe asked for by a key or a button, which the stack plays on its top
/// card. `Copy`, so the host's key handler and buttons share it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SwipeControl(Signal<Option<Swipe>>);

impl SwipeControl {
    /// Asks the stack to swipe its top card `swipe`.
    pub fn press(mut self, swipe: Swipe) {
        self.0.set(Some(swipe));
    }
}

/// Where a key or button press goes on a surface whose pile can run out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwipeRoute {
    /// To the stack, which plays it on its top card.
    Stack,
    /// Straight to the host's undo.
    Undo,
    /// Nowhere, since no card is on top to take it.
    Ignore,
}

impl SwipeRoute {
    /// Routes `swipe` by whether a card is on top. With none, down goes
    /// straight to undo and never through [`SwipeControl`], where it would
    /// wait for the next stack to mount and swipe its first card.
    pub fn of(swipe: Swipe, card_on_top: bool) -> Self {
        match (card_on_top, swipe) {
            (true, _) => Self::Stack,
            (false, Swipe::Down) => Self::Undo,
            (false, _) => Self::Ignore,
        }
    }
}

/// A control with nothing asked for yet.
pub fn use_swipe_control() -> SwipeControl {
    SwipeControl(use_signal(|| None))
}

/// One committed card on its way off the stack.
#[derive(Debug, Clone, PartialEq)]
struct Exiting {
    key: Uuid,
    card: Card,
    direction: Direction,
    from_x: f64,
    from_y: f64,
}

/// Where a drag started and where it is now, in window pixels.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct Drag {
    start: (f64, f64),
    now: (f64, f64),
}

impl Drag {
    fn delta(self) -> (f64, f64) {
        (self.now.0 - self.start.0, self.now.1 - self.start.1)
    }
}

/// The stack: `cards` top first, of which only the top takes input. A swipe
/// in `allowed` calls `on_swipe` with the top card; any other springs back.
/// `entering` names the side the top card comes back from after an undo,
/// and `on_entered` clears it once that keyframe ends. `on_top_shown` fires
/// once the top card's image has its bytes, failed, or has no art. Until
/// then, and for [`TOP_PAINT_GRACE`] after when it came in on top, the
/// stack draws `ghost` under its cards and hides the cards under the top,
/// so neither an empty slot nor another card shows while the top's image
/// arrives. `hold` keeps the ghost up regardless, for a pile the host has
/// yet to show.
#[component]
pub fn SwipeStack(
    cards: Vec<Card>,
    allowed: Vec<Swipe>,
    control: SwipeControl,
    on_swipe: EventHandler<(Swipe, Card)>,
    #[props(default)] entering: Option<Direction>,
    #[props(default)] on_entered: Option<EventHandler<()>>,
    #[props(default)] on_top_shown: Option<EventHandler<()>>,
    #[props(default)] ghost: Option<Element>,
    #[props(default)] hold: bool,
) -> Element {
    let mut drag: Signal<Option<Drag>> = use_signal(|| None);
    // The cards whose image has loaded and decoded, failed, or has no art,
    // and the top whose image came in while it was the top, while it paints.
    let mut shown_cards: Signal<HashSet<Uuid>> = use_signal(HashSet::new);
    let mut painting: Signal<Option<Uuid>> = use_signal(|| None);
    let mut exiting: Signal<Vec<Exiting>> = use_signal(Vec::new);
    // When the last card started flying off, on the browser's clock.
    let mut last_exit_at: Signal<Option<DateTime<Utc>>> = use_signal(|| None);
    let top = cards.first().cloned();
    let allowed_for_control = allowed.clone();

    let commit = use_callback(move |(swipe, card, from): (Swipe, Card, (f64, f64))| {
        if let Some(direction) = swipe.exit() {
            last_exit_at.set(Some(Utc::now()));
            exiting.write().push(Exiting {
                key: Uuid::new_v4(),
                card: card.clone(),
                direction,
                from_x: from.0,
                from_y: from.1,
            });
        }
        on_swipe.call((swipe, card));
    });

    // A key or a button: play the swipe on the top card from rest, or drop
    // it while the last card is less than half way out.
    let mut asked = control.0;
    use_effect(use_reactive!(|top| {
        let Some(swipe) = asked() else {
            return;
        };
        asked.set(None);
        let last_exit = *last_exit_at.peek();
        let since_last_exit =
            last_exit.map(|at| Utc::now().signed_duration_since(at).num_milliseconds());
        if !allowed_for_control.contains(&swipe) || !swipe.plays_after(since_last_exit) {
            return;
        }
        if let Some(card) = top {
            commit.call((swipe, card, (0.0, 0.0)));
        }
    }));

    let shown: Vec<Card> = cards.into_iter().take(STACK_DEPTH).collect();
    let count = shown.len();
    let allowed_up = allowed.contains(&Swipe::Up);
    let top_id = shown.first().map(|c| c.scryfall_data.id);
    let held = hold || ghost_held(top_id, &shown_cards.read(), painting());
    // A top whose image came in while it was the top has the grace to paint
    // before the ghost goes and the cards under it may show.
    let painting_now = painting();
    use_effect(use_reactive!(|painting_now| {
        if let Some(id) = painting_now {
            spawn(async move {
                timer::sleep(TOP_PAINT_GRACE).await;
                if *painting.peek() == Some(id) {
                    painting.set(None);
                }
            });
        }
    }));

    let layers = shown.into_iter().enumerate().rev().map(|(i, card)| {
        let is_top = i == 0;
        let z = count.saturating_sub(i);
        let style = if is_top {
            let (dx, dy) = drag().map(Drag::delta).unwrap_or_default();
            let horizontal = dx.abs() >= dy.abs();
            let tx = if horizontal { dx } else { 0.0 };
            let ty = if !horizontal && allowed_up {
                dy.min(0.0)
            } else {
                0.0
            };
            let rot = tx * TILT_PER_PX;
            let transition = if drag().is_some() {
                "transition: transform 0s;"
            } else {
                "transition: transform 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);"
            };
            format!(
                "transform: translate({tx}px, {ty}px) rotate({rot}deg); {transition} z-index: {z};"
            )
        } else {
            format!("z-index: {z};")
        };
        let entering_class = match (is_top, entering) {
            (true, Some(Direction::Left)) => " card-stack-enter-left",
            (true, Some(Direction::Right)) => " card-stack-enter-right",
            (true, Some(Direction::Up)) => " card-stack-enter-up",
            _ => "",
        };
        let id = card.scryfall_data.id;
        Layer {
            id,
            name: card.scryfall_data.name.clone(),
            is_top,
            class: format!(
                "swipe-stack-card{}{entering_class}",
                if is_top { " top" } else { "" }
            ),
            style,
            loaded: shown_cards.read().contains(&id),
            sd: card.scryfall_data.clone(),
            card,
            allowed: allowed.clone(),
        }
    });

    rsx! {
        div { class: if held { "swipe-stack web-swipe-stack ghost-held" } else { "swipe-stack web-swipe-stack" },
            if held && let Some(ghost) = ghost {
                div { class: "swipe-stack-ghost", {ghost} }
            }
            // Keyed by card, the loop's own node, so a swipe drops the top
            // and keeps every other card mounted.
            for layer in layers {
                div {
                    key: "{layer.id}",
                    class: "{layer.class}",
                    style: "{layer.style}",
                    "aria-label": "{layer.name}",
                    onpointerdown: move |e: Event<PointerData>| {
                        if !layer.is_top || entering.is_some() {
                            return;
                        }
                        let p = e.client_coordinates();
                        drag.set(Some(Drag { start: (p.x, p.y), now: (p.x, p.y) }));
                    },
                    onpointermove: move |e: Event<PointerData>| {
                        let current = *drag.peek();
                        if let Some(mut d) = current {
                            let p = e.client_coordinates();
                            d.now = (p.x, p.y);
                            drag.set(Some(d));
                        }
                    },
                    onpointerup: move |_| {
                        let Some(d) = drag.take() else {
                            return;
                        };
                        let (dx, dy) = d.delta();
                        if let Some(swipe) = Swipe::of_drag(dx, dy).filter(|s| layer.allowed.contains(s)) {
                            let from = if swipe == Swipe::Up { (0.0, dy.min(0.0)) } else { (dx, 0.0) };
                            commit.call((swipe, layer.card.clone(), from));
                        }
                    },
                    onpointerleave: move |_| drag.set(None),
                    onpointercancel: move |_| drag.set(None),
                    onanimationend: move |_| {
                        if layer.is_top && entering.is_some()
                            && let Some(handler) = on_entered
                        {
                            handler.call(());
                        }
                    },
                    components::FlippableCardImage {
                        sd: layer.sd,
                        size: ImageSize::Large,
                        class: "card-image".to_string(),
                        draggable: false,
                        flippable: layer.is_top,
                        // Shows at once if this stack already had it in, else
                        // only from its own load.
                        trust_seen: layer.loaded,
                        on_load: move |()| {
                            if layer.is_top && !shown_cards.peek().contains(&layer.id) {
                                painting.set(Some(layer.id));
                            }
                            shown_cards.write().insert(layer.id);
                            if layer.is_top && let Some(handler) = on_top_shown {
                                handler.call(());
                            }
                        },
                    }
                }
            }
            for gone in exiting() {
                ExitingCard {
                    key: "{gone.key}",
                    gone,
                    on_end: move |key: Uuid| exiting.write().retain(|e| e.key != key),
                }
            }
        }
    }
}

/// One card of the stack as it draws, worked out before the markup so the
/// markup's loop can key each card by its id.
struct Layer {
    id: Uuid,
    name: String,
    is_top: bool,
    class: String,
    style: String,
    /// Whether this stack already had the card's image in.
    loaded: bool,
    sd: ScryfallData,
    card: Card,
    allowed: Vec<Swipe>,
}

/// One card flying off on its exit keyframe, keyed by its own id so the
/// cards still flying keep their place when another one lands. `on_end`
/// takes the id once the flight ends.
#[component]
fn ExitingCard(gone: Exiting, on_end: EventHandler<Uuid>) -> Element {
    let exit_class = match gone.direction {
        Direction::Left => "card-stack-exit-left",
        Direction::Right => "card-stack-exit-right",
        Direction::Up => "card-stack-exit-up",
    };
    let key = gone.key;
    let rot = gone.from_x * TILT_PER_PX;
    rsx! {
        div {
            class: "swipe-stack-card swipe-stack-exiting {exit_class}",
            style: "z-index: 100; --exit-from-x: {gone.from_x}px; --exit-from-y: {gone.from_y}px; --exit-from-rot: {rot}deg;",
            onanimationend: move |_| on_end.call(key),
            components::FlippableCardImage {
                sd: gone.card.scryfall_data.clone(),
                size: ImageSize::Large,
                class: "card-image".to_string(),
                draggable: false,
                flippable: false,
            }
        }
    }
}

/// The swipes as an action dock beside the surface's own dock, which comes
/// in as `children`: one button per direction in `buttons`, each with its
/// arrow and the action's name, the name hiding below the breakpoint. The
/// two docks sit side by side at the bottom right on a wide window and the
/// swipes' stack over the other below it. The buttons route as the arrow
/// keys do ([`SwipeRoute::of`]): through `control` while `card_on_top`,
/// else down calls `on_undo` and the rest are off.
#[component]
pub fn SwipeDock(
    control: SwipeControl,
    buttons: Vec<(Swipe, &'static str)>,
    card_on_top: bool,
    on_undo: EventHandler<()>,
    children: Element,
) -> Element {
    rsx! {
        div { class: "web-dock-pair",
            div { class: "action-dock web-swipe-dock",
                for (swipe, label) in buttons {
                    {
                        let route = SwipeRoute::of(swipe, card_on_top);
                        rsx! {
                            button {
                                key: "{label}",
                                class: "action-dock-btn web-swipe-dock-btn",
                                "aria-label": label,
                                title: label,
                                disabled: route == SwipeRoute::Ignore,
                                onclick: move |_| match route {
                                    SwipeRoute::Stack => control.press(swipe),
                                    SwipeRoute::Undo => on_undo.call(()),
                                    SwipeRoute::Ignore => {}
                                },
                                span { class: "web-swipe-arrow", DockIcon { path: swipe.arrow() } }
                                span { class: "web-swipe-dock-label", "{label}" }
                            }
                        }
                    }
                }
            }
            {children}
        }
    }
}

/// Presses the top swipe card's flip control, which a card with two faces
/// has, else its Meld toggle, which a meld half has; on any other card
/// nothing happens.
pub fn flip_top_card() {
    document::eval(&shortcuts::press_first_js(
        ".swipe-stack-card.top",
        &components::data_action::FLIP_OR_MELD,
    ));
}

/// Listens for arrow keys like [`use_swipe_keys`], routed by
/// [`SwipeRoute::of`]: through `control` while `card_on_top` says a card is
/// on top, else down calls `on_undo` and the rest do nothing.
pub fn use_swipe_keys_or_undo(
    control: SwipeControl,
    keys: Vec<Swipe>,
    card_on_top: Callback<(), bool>,
    on_undo: Callback<()>,
) {
    use_swipe_key_listener(
        keys,
        Letters::Vim,
        Callback::new(
            move |swipe: Swipe| match SwipeRoute::of(swipe, card_on_top.call(())) {
                SwipeRoute::Stack => control.press(swipe),
                SwipeRoute::Undo => on_undo.call(()),
                SwipeRoute::Ignore => {}
            },
        ),
    );
}

/// Listens for the arrow keys in `keys` while this surface is the top one
/// and no text field or dialog has focus, and presses them on `control`.
/// Surfaces stack: one mounted over another takes the keys until it goes.
pub fn use_swipe_keys(control: SwipeControl, keys: Vec<Swipe>) {
    use_swipe_key_handler(keys, Callback::new(move |swipe| control.press(swipe)));
}

/// [`use_swipe_keys`] that hands each key to `on_key` instead, for a host
/// that answers a key itself when no card is on top, such as undo at the
/// end of the results.
pub fn use_swipe_key_handler(keys: Vec<Swipe>, on_key: Callback<Swipe>) {
    use_swipe_key_listener(keys, Letters::Arrows, on_key);
}

/// Which keys stand for a swipe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Letters {
    /// The arrow keys alone.
    Arrows,
    /// The arrow keys and the swipe screens' `hjkl` and `u`
    /// ([`shortcuts::swipe_screen_shortcut`]).
    Vim,
}

/// The keys that swipe in `keys`' directions under `letters`.
fn swipe_keys(keys: &[Swipe], letters: Letters) -> Vec<&'static str> {
    let vim: &[&'static str] = match letters {
        Letters::Arrows => &[],
        Letters::Vim => &shortcuts::SWIPE_LETTERS,
    };
    keys.iter()
        .map(|s| s.key())
        .chain(vim.iter().copied().filter(|key| {
            matches!(shortcuts::swipe_screen_shortcut(key), Some(Shortcut::Swipe(s)) if keys.contains(&s))
        }))
        .collect()
}

/// Calls `on_key` with the swipe each of `keys`' keys stands for, under
/// `letters`, while this surface is the newest and the key reaches it.
fn use_swipe_key_listener(keys: Vec<Swipe>, letters: Letters, on_key: Callback<Swipe>) {
    let id = use_hook(Uuid::new_v4);
    use_hook(move || {
        let wanted = swipe_keys(&keys, letters);
        let script = SWIPE_KEYS_JS
            .replace("__ID__", &id.to_string())
            .replace("__KEYS__", &format!("{wanted:?}"))
            .replace("__GUARD__", &shortcuts::guard_js());
        spawn(async move {
            let mut keys_in = document::eval(&script);
            while let Ok(key) = keys_in.recv::<String>().await {
                if let Some(Shortcut::Swipe(swipe)) = shortcuts::swipe_screen_shortcut(&key) {
                    on_key.call(swipe);
                }
            }
        });
    });
    use_drop(move || {
        document::eval(&SWIPE_KEYS_DROP_JS.replace("__ID__", &id.to_string()));
    });
}

/// Registers a key listener for one surface. Only the newest registered
/// surface answers, never while a text field has focus or a dialog is open,
/// and never to a held key's repeats.
const SWIPE_KEYS_JS: &str = r"
window.__zwipeSwipe = window.__zwipeSwipe || [];
window.__zwipeSwipe.push('__ID__');
const wanted = __KEYS__;
const onKey = (e) => {
    const stack = window.__zwipeSwipe;
    if (!stack.length || stack[stack.length - 1] !== '__ID__') return;
    if (!wanted.includes(e.key) || !__GUARD__) return;
    e.preventDefault();
    if (e.repeat) return;
    dioxus.send(e.key);
};
window['__zwipeSwipeKey__ID__'] = onKey;
document.addEventListener('keydown', onKey);
";

/// Removes one surface's key listener.
const SWIPE_KEYS_DROP_JS: &str = r"
const onKey = window['__zwipeSwipeKey__ID__'];
if (onKey) document.removeEventListener('keydown', onKey);
delete window['__zwipeSwipeKey__ID__'];
window.__zwipeSwipe = (window.__zwipeSwipe || []).filter((id) => id !== '__ID__');
";

#[cfg(test)]
mod tests {
    use super::*;

    /// The swipe screens' shared sheet, as the web links it.
    const SCREENS_CSS: &str = include_str!("../../../../assets/sheets/07-screens.css");

    #[test]
    fn a_card_flying_off_shows_while_the_stack_holds_its_ghost() {
        let rule = SCREENS_CSS
            .lines()
            .find(|line| line.starts_with(".swipe-stack.ghost-held > .swipe-stack-card"))
            .expect("the rule hiding the cards under a held top");
        assert!(
            rule.contains(".swipe-stack-exiting"),
            "the exiting card is exempt, or every fly-off is invisible: {rule}"
        );
    }

    #[test]
    fn u_undoes_with_the_vim_letters() {
        let down = swipe_keys(&[Swipe::Down], Letters::Vim);
        for key in ["j", "J", "u", "U"] {
            assert!(down.contains(&key), "{key} in {down:?}");
        }
        assert!(
            !swipe_keys(&[Swipe::Down], Letters::Arrows).contains(&"u"),
            "a surface without the letters keeps u off"
        );
    }

    #[test]
    fn a_short_drag_springs_back() {
        assert_eq!(Swipe::of_drag(89.0, 0.0), None);
        assert_eq!(Swipe::of_drag(0.0, -89.0), None);
    }

    #[test]
    fn a_card_on_top_takes_every_press() {
        for swipe in [Swipe::Left, Swipe::Right, Swipe::Up, Swipe::Down] {
            assert_eq!(SwipeRoute::of(swipe, true), SwipeRoute::Stack, "{swipe:?}");
        }
    }

    #[test]
    fn an_empty_pile_undoes_on_down_and_ignores_the_rest() {
        assert_eq!(SwipeRoute::of(Swipe::Down, false), SwipeRoute::Undo);
        for swipe in [Swipe::Left, Swipe::Right, Swipe::Up] {
            assert_eq!(
                SwipeRoute::of(swipe, false),
                SwipeRoute::Ignore,
                "{swipe:?}"
            );
        }
    }

    #[test]
    fn a_press_waits_until_the_last_card_is_half_way_out() {
        for swipe in [Swipe::Left, Swipe::Right, Swipe::Up] {
            assert!(swipe.plays_after(None), "{swipe:?}");
            assert!(!swipe.plays_after(Some(0)), "{swipe:?}");
            assert!(!swipe.plays_after(Some(PRESS_GAP_MS - 1)), "{swipe:?}");
            assert!(swipe.plays_after(Some(PRESS_GAP_MS)), "{swipe:?}");
            assert!(swipe.plays_after(Some(EXIT_MS)), "{swipe:?}");
        }
    }

    #[test]
    fn undo_never_waits() {
        assert!(Swipe::Down.plays_after(None));
        assert!(Swipe::Down.plays_after(Some(0)));
    }

    #[test]
    fn the_longer_axis_decides() {
        assert_eq!(Swipe::of_drag(120.0, 100.0), Some(Swipe::Right));
        assert_eq!(Swipe::of_drag(-120.0, 30.0), Some(Swipe::Left));
        assert_eq!(Swipe::of_drag(40.0, -95.0), Some(Swipe::Up));
        assert_eq!(Swipe::of_drag(-20.0, 95.0), Some(Swipe::Down));
    }
}
