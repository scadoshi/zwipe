//! The card search bar over zwipe-ui's [`model`], the same in every host: the box, the ✕ once
//! it holds text, and the matches as chips under it, a line in their place while a lookup is
//! out, matches nothing, or fails.
//!
//! The glass is the box's label chip, just left of it. On a wide screen the bar is the nav's
//! box: the chip and 120px of box at rest, the chip alone where the nav leaves less room, the
//! box growing over the nav's links while it has focus, with the chips in a dropdown under it.
//! Where the host's header folds (`header_media`), the chip sits alone in the header, and a tap
//! on it fades the header's other items out and opens the box across the header, the chips
//! dropping under it over the page; the ✕ or Escape gives the header back. `/` focuses the box,
//! unless a text field has focus or the screen answers its own keys.
//!
//! A host runs the model's effects through [`use_card_search`] with its own [`SearchHost`]: its
//! clock, its lookup and its routes.

use std::future::Future;
use std::time::Duration;

use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::http::contracts::card::HttpCardLookup;
use zwipe_ui::screens::card_search::{
    self as model, Action, Destination, Effect, Failure, State, Status, Step,
};
use zwipe_ui::shortcuts;

use crate::ClearWrap;
use crate::keys::use_page_keys;

/// The search icon: a lens and its handle, Lucide's `search` drawn as one path (ISC,
/// `components/LICENSE-LUCIDE`).
pub const SEARCH_ICON: &str = "M11 3a8 8 0 1 0 0 16 8 8 0 1 0 0-16M21 21l-4.3-4.3";

/// The nav box's width while it has focus, when the nav has that much room before its links.
pub const NAV_GROWN_PX: u32 = 320;

/// The room kept between the grown box and the first link it leaves showing.
const NAV_GAP_PX: u32 = 16;

/// The least room before the first link the box grows into without covering the links.
const NAV_ROOM_MIN_PX: u32 = 240;

/// Focuses the first card search box on the page that shows.
const FOCUS_SHOWN_JS: &str = "[...document.querySelectorAll('.card-search-input')].find((el) => el.offsetParent !== null)?.focus();";

/// Answers whether any card search box shows on the page for `/` to focus, the test
/// [`FOCUS_SHOWN_JS`] focuses by.
pub(crate) const ANY_SHOWN_JS: &str = "return [...document.querySelectorAll('.card-search-input')].some((el) => el.offsetParent !== null);";

/// What a host does for the bar: waits, looks up and opens.
pub trait SearchHost: Copy + 'static {
    /// Resolves after `wait`.
    fn sleep(self, wait: Duration) -> impl Future<Output = ()>;

    /// Sends `lookup`, with the session's token when one is held.
    ///
    /// # Errors
    ///
    /// The model's [`Failure`]: rate limited with the server's wait, or any other failure.
    fn lookup(self, lookup: HttpCardLookup) -> impl Future<Output = Result<Vec<Card>, Failure>>;

    /// Opens `to`: a card page or the results page, as the host's route or link.
    fn open(self, to: Destination);
}

/// The bar's model and the host that performs its effects. Copy, so every handler and task
/// holds its own.
#[derive(Clone, Copy)]
pub struct CardSearch<H> {
    model: Signal<State>,
    host: H,
}

impl<H> std::fmt::Debug for CardSearch<H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CardSearch").finish_non_exhaustive()
    }
}

impl<H: SearchHost> CardSearch<H> {
    /// The model, for the bar to draw.
    #[must_use]
    pub fn model(self) -> Signal<State> {
        self.model
    }

    /// Applies `action` to the model and performs what it returns.
    pub fn dispatch(self, action: Action) {
        let mut model = self.model;
        let effects = model.write().update(action);
        for effect in effects {
            self.perform(effect);
        }
    }

    fn perform(self, effect: Effect) {
        match effect {
            Effect::Debounce { ticket, wait } => {
                spawn(async move {
                    self.host.sleep(wait).await;
                    self.dispatch(Action::Debounced(ticket));
                });
            }
            Effect::Lookup { ticket, lookup } => {
                spawn(async move {
                    let reply = match self.host.lookup(lookup).await {
                        Ok(cards) => Action::Loaded { ticket, cards },
                        Err(failure) => Action::Failed { ticket, failure },
                    };
                    self.dispatch(reply);
                });
            }
            Effect::Cooldown { ticket, wait } => {
                spawn(async move {
                    self.host.sleep(wait).await;
                    self.dispatch(Action::CooledDown(ticket));
                });
            }
            Effect::Navigate(to) => self.host.open(to),
        }
    }
}

/// Puts the cursor in the card search box that shows, for `/` on a screen whose own keys answer
/// before the bar's.
pub fn focus_card_search() {
    document::eval(FOCUS_SHOWN_JS);
}

/// A fresh bar driven by `host`, for one mount of the bar.
pub fn use_card_search<H: SearchHost>(host: H) -> CardSearch<H> {
    CardSearch {
        model: use_signal(State::new),
        host,
    }
}

/// The bar: the glass's label chip, the box beside it and what shows under it. `input_id` names
/// the box, unique on the page. `on_action` takes everything the bar does, for
/// [`CardSearch::dispatch`]. `header_media` is the media query under which the bar is the
/// header's search ([`HEADER_ALWAYS`] in the app, empty where it never is): the chip alone in
/// the header until a tap opens the box across it.
#[component]
#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "keyboard_types::Key is a foreign enum of every key; only the keys handled act"
)]
pub fn CardSearchBar(
    model: Signal<State>,
    on_action: Callback<Action>,
    input_id: String,
    #[props(default)] header_media: String,
) -> Element {
    // `/` puts the cursor in the card search, while no field has focus and the screen answers no
    // keys of its own; a screen that does registers after the bar, so its keys win.
    use_page_keys(Callback::new(move |key: String| {
        if key == shortcuts::CARD_SEARCH_KEY {
            focus_card_search();
        }
    }));
    let header = use_header_media(&header_media);

    let state = model.read();
    let query = state.query().to_owned();
    let status = state.status();
    let lit = state.highlight();
    let results: Vec<(Uuid, String)> = state
        .results()
        .iter()
        .map(|c| (c.scryfall_data.id, c.scryfall_data.name.clone()))
        .collect();
    drop(state);
    let line = match status {
        Status::Idle | Status::Results => None,
        Status::Loading => Some(model::SEARCHING),
        Status::NoMatch => Some(model::NO_MATCH),
        Status::RateLimited => Some(model::RATE_LIMITED),
        Status::Failed => Some(model::FAILED),
    };
    let filled = !query.is_empty();
    let blur_id = input_id.clone();
    // Letting the box go shrinks it back over the links, or gives the header back.
    let let_go = use_callback(move |()| {
        document::eval(&blur_js(&blur_id));
    });
    let label_for = input_id.clone();
    let grow_id = format!("{input_id}-box");
    let box_id = grow_id.clone();

    let field = rsx! {
        ClearWrap {
            filled,
            // The header's ✕ also gives the header back.
            on_clear: move |()| {
                on_action.call(Action::Clear);
                if header() {
                    let_go.call(());
                }
            },
            label: "Clear the search".to_string(),
            input {
                class: "input card-search-input",
                id: "{input_id}",
                r#type: "text",
                placeholder: model::PLACEHOLDER,
                "aria-label": model::NAME,
                value: "{query}",
                autocapitalize: "none",
                autocorrect: "off",
                autocomplete: "off",
                spellcheck: "false",
                enterkeyhint: "search",
                oninput: move |event| on_action.call(Action::Type(event.value())),
                onkeydown: move |event: Event<KeyboardData>| {
                    let action = match event.key() {
                        Key::ArrowDown => Action::Step(Step::Down),
                        Key::ArrowUp => Action::Step(Step::Up),
                        Key::Enter => Action::Enter,
                        Key::Escape => {
                            event.prevent_default();
                            event.stop_propagation();
                            let empty = model.peek().query().is_empty();
                            if !empty {
                                on_action.call(Action::Clear);
                            }
                            if escape_lets_go(empty, header()) {
                                let_go.call(());
                            }
                            return;
                        }
                        _ => return,
                    };
                    event.prevent_default();
                    let enter = action == Action::Enter;
                    on_action.call(action);
                    // An Enter that opened something emptied the box.
                    if enter && model.peek().query().is_empty() {
                        let_go.call(());
                    }
                },
            }
        }
    };
    let shows_under = line.is_some() || !results.is_empty();
    let under = rsx! {
        if let Some(line) = line {
            span { class: "chip-unselected card-search-line", {line} }
            if status == Status::Failed {
                button {
                    r#type: "button",
                    class: "chip-unselected card-search-chip",
                    onmousedown: move |event| event.prevent_default(),
                    onclick: move |_| on_action.call(Action::Retry),
                    "Retry"
                }
            }
        } else {
            for (i, (id, name)) in results.into_iter().enumerate() {
                button {
                    key: "{id}",
                    r#type: "button",
                    class: if lit == Some(i) { "chip-unselected card-search-chip card-search-lit" } else { "chip-unselected card-search-chip" },
                    style: "animation-delay: {i.saturating_mul(40)}ms;",
                    // The box keeps focus, so the dropdown stays open under the tap.
                    onmousedown: move |event| event.prevent_default(),
                    onclick: move |_| {
                        on_action.call(Action::Pick(id));
                        let_go.call(());
                    },
                    "{name}"
                }
            }
        }
    };

    let mut class = String::from("card-search card-search-nav");
    if filled {
        class.push_str(" has-text");
    }
    if header() {
        class.push_str(" card-search-header");
    }
    rsx! {
        div { class: "{class}",
            // The glass is the box's label chip: a tap on it puts the cursor in the box, which
            // opens the box in the same tap, so a phone raises its keyboard. Where there is room
            // only for an icon, it shows alone.
            label {
                class: "stat-chip stat-chip-p1 chip-row-label card-search-label",
                r#for: "{label_for}",
                "aria-label": model::NAME,
                svg {
                    class: "card-search-icon",
                    view_box: "0 0 24 24",
                    xmlns: "http://www.w3.org/2000/svg",
                    "aria-hidden": "true",
                    path { d: SEARCH_ICON }
                }
            }
            div {
                class: "card-search-slot",
                id: "{box_id}",
                onfocusin: move |_| {
                    if !header() {
                        document::eval(&grow_js(&grow_id));
                    }
                },
                div { class: "card-search-field", {field} }
                if filled && shows_under {
                    div { class: "card-search-results card-search-drop", {under} }
                }
            }
        }
    }
}

/// The `header_media` that makes the bar the header's search at every width, as in the app.
pub const HEADER_ALWAYS: &str = "all";

/// Whether the window matches `media`, kept up to date as it changes: always for
/// [`HEADER_ALWAYS`], never for an empty query.
fn use_header_media(media: &str) -> Signal<bool> {
    let media = media.to_owned();
    let mut matches = use_signal(|| media == HEADER_ALWAYS);
    let id = use_hook(Uuid::new_v4);
    use_hook(move || {
        if media.is_empty() || media == HEADER_ALWAYS {
            return;
        }
        let script = media_js(&media, id);
        spawn(async move {
            let mut changes = document::eval(&script);
            while let Ok(now) = changes.recv::<bool>().await {
                matches.set(now);
            }
        });
    });
    use_drop(move || {
        document::eval(&media_drop_js(id));
    });
    matches
}

/// Sends whether `media` matches, now and on every change, under listener `id`.
fn media_js(media: &str, id: Uuid) -> String {
    // A host's own constant; its debug form escapes quotes and backslashes as a JS string does.
    let literal = format!("{media:?}");
    format!(
        r"
const q = window.matchMedia({literal});
const on = (e) => dioxus.send(e.matches);
window['__zwipeSearchMedia{id}'] = [q, on];
q.addEventListener('change', on);
dioxus.send(q.matches);
"
    )
}

/// Stops listener `id` of [`media_js`].
fn media_drop_js(id: Uuid) -> String {
    format!(
        "const held = window['__zwipeSearchMedia{id}']; if (held) {{ held[0].removeEventListener('change', held[1]); delete window['__zwipeSearchMedia{id}']; }}"
    )
}

/// Whether Escape lets the box go: on an empty box, and always in the header, where it gives the
/// header back. A nav box with text keeps the cursor once Escape empties it, so the next letter
/// starts a new search.
fn escape_lets_go(empty: bool, header: bool) -> bool {
    empty || header
}

/// Sizes the nav box with id `box_id` for focus: [`NAV_GROWN_PX`] when the nav has that much
/// room before its first link on the box's row, all of that room when it is at least
/// [`NAV_ROOM_MIN_PX`], else out to the last link's end, over the links, marking the nav so the
/// links it covers fade.
fn grow_js(box_id: &str) -> String {
    format!(
        r"(() => {{
const box = document.getElementById('{box_id}');
if (!box) return;
const nav = box.closest('nav');
const items = nav ? [...nav.querySelectorAll('.nav-links > li')] : [];
const first = items[0];
const last = items[items.length - 1];
const b = box.getBoundingClientRect();
let grown = {NAV_GROWN_PX};
let covers = false;
if (first && last) {{
    const f = first.getBoundingClientRect();
    const l = last.getBoundingClientRect();
    const room = Math.round(f.left - {NAV_GAP_PX} - b.left);
    const sameRow = f.top < b.bottom && f.bottom > b.top;
    if (sameRow && room < {NAV_GROWN_PX}) {{
        if (room >= {NAV_ROOM_MIN_PX}) {{
            grown = room;
        }} else {{
            grown = Math.max(Math.round(l.right - b.left), b.width);
            covers = true;
        }}
    }}
}}
box.style.setProperty('--card-search-grown', grown + 'px');
if (nav) nav.classList.toggle('card-search-covers', covers);
}})()"
    )
}

/// Takes focus off the box with id `input_id`, and off the bar's ✕ when a tap left it there.
fn blur_js(input_id: &str) -> String {
    format!(
        "document.getElementById('{input_id}')?.blur(); const held = document.activeElement; if (held && held.closest('.card-search')) held.blur();"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_grow_script_names_the_box_and_the_grown_width() {
        let js = grow_js("nav-card-search-box");
        assert!(js.contains("getElementById('nav-card-search-box')"), "{js}");
        assert!(js.contains("let grown = 320;"), "{js}");
        assert!(js.contains("f.left - 16 - b.left"), "{js}");
        assert!(js.contains("room < 320"), "{js}");
        assert!(js.contains("room >= 240"), "{js}");
        assert!(js.contains("card-search-covers"), "{js}");
    }

    #[test]
    fn escape_lets_an_empty_box_go_and_always_gives_the_header_back() {
        assert!(escape_lets_go(true, false), "an empty nav box");
        assert!(
            !escape_lets_go(false, false),
            "a nav box with text keeps the cursor"
        );
        assert!(escape_lets_go(false, true), "the header's box with text");
        assert!(escape_lets_go(true, true), "the header's empty box");
    }

    #[test]
    fn the_media_script_reports_the_query_now_and_on_change() {
        let id = Uuid::from_u128(7);
        let js = media_js("(max-width: 1179px)", id);
        assert!(
            js.contains(r#"window.matchMedia("(max-width: 1179px)")"#),
            "{js}"
        );
        assert!(js.contains("dioxus.send(q.matches)"), "{js}");
        assert!(js.contains("addEventListener('change', on)"), "{js}");
        assert!(media_drop_js(id).contains("removeEventListener('change'"));
    }

    #[test]
    fn the_blur_script_lets_go_of_the_box_and_of_its_clear() {
        let js = blur_js("nav-card-search");
        assert!(
            js.starts_with("document.getElementById('nav-card-search')?.blur();"),
            "{js}"
        );
        assert!(
            js.contains("held.closest('.card-search')"),
            "the ✕ a tap focused lets go too: {js}"
        );
    }
}
