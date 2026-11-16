//! Stacked card swipe component.
//!
//! Renders up to [`STACK_DEPTH`] cards from a slice as a peeking stack. Only
//! the top card tracks gestures. When the top card commits a swipe, two things
//! happen *simultaneously*:
//!
//! 1. The card is pushed onto an internal `exiting_overlay` and rendered as a
//!    detached layer above the stack, playing a one-shot CSS keyframe that
//!    flies it off-screen and removes itself on `animationend`.
//! 2. The parent's `on_swipe_*` callback fires immediately so `current_index`
//!    advances right away: the card beneath becomes the new top on the very
//!    next render and is interactive without delay.
//!
//! Down-swipes do not exit. They fire `on_swipe_down` for undo and the active
//! card is visually clamped (Y delta cannot go positive) so it never moves on
//! screen during the gesture.
//!
//! Undo uses the `entering` signal: when set to `Some(dir)`, the freshly-
//! restored top card plays a keyframe animation entering from `dir`, then the
//! stack clears the signal on `animationend`, or as soon as a gesture starts
//! on the card, which ends the return with the card at rest.

use std::collections::{HashMap, HashSet};

use components::{GhostCard, share_face, use_shared_face};
use dioxus::html::geometry::euclid::{Point2D, UnknownUnit};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::card_profile::CardProfile;
use zwipe_core::domain::card::scryfall_data::{ImageSize, ScryfallData};
use zwipe_ui::stack::ghost_held;
use zwipe_ui::timing::TOP_PAINT_GRACE;

use crate::inbound::components::interactions::gesture_math;
use crate::inbound::components::interactions::swipe::axis::Axis;
use crate::inbound::components::interactions::swipe::config::SwipeConfig;
use crate::inbound::components::interactions::swipe::direction::Direction;
use crate::inbound::components::interactions::swipe::onmouse::OnMouse;
use crate::inbound::components::interactions::swipe::ontouch::OnTouch;
use crate::inbound::components::interactions::swipe::state::SwipeState;
use crate::inbound::screens::deck::card::components::flippable_card_image::FlippableCardImage;

type DeltaPoint = Point2D<f64, UnknownUnit>;

/// How many cards are rendered at once (top + peeking underneath).
pub const STACK_DEPTH: usize = 10;

/// Vertical offset per peek layer in pixels.
const PEEK_OFFSET_PX: f64 = 0.0;
/// Scale reduction per peek layer.
const PEEK_SCALE_STEP: f64 = 0.0;
/// Degrees of rotation applied per pixel of horizontal drag (the "tilt").
const TILT_PER_PX: f64 = 0.06;

/// One committed card mid-exit. The seed fields carry the transform the card
/// had at the moment the finger released it: the exit keyframe starts from
/// there (via CSS custom properties) instead of snapping back to rest first.
#[derive(Clone, PartialEq)]
struct ExitingCard {
    id: Uuid,
    direction: Direction,
    card: Card,
    /// Horizontal offset at release, px.
    from_x: f64,
    /// Vertical offset at release, px.
    from_y: f64,
    /// Tilt at release, degrees.
    from_rot: f64,
}

/// The stack's swipe callbacks, one per direction.
#[derive(Clone, Copy)]
struct SwipeHandlers {
    left: EventHandler<Card>,
    right: EventHandler<Card>,
    up: EventHandler<Card>,
    down: EventHandler<Card>,
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
    /// The card's roles and tags, which a no-art proxy lists.
    profile: CardProfile,
    /// The stack's top card and the config a touch release dispatches, on every card: a
    /// gesture outlives its card's turn on top when the frame for an earlier undo lands
    /// mid-swipe, and its release still acts on the top.
    touch: Option<(Card, SwipeConfig)>,
    /// The top card and the config a mouse release dispatches, on every card.
    mouse: Option<(Card, SwipeConfig)>,
}

/// A finger or mouse on the stack while the top card comes back from an undo: the return ends
/// now, the card at rest under the finger, so the gesture that follows counts, on whichever card
/// it started. A down swipe undoes again; any other swipe takes the top card, and the card under
/// it comes up without a return of its own.
fn take_hold(entering: &mut Signal<Option<Direction>>) {
    if entering.peek().is_some() {
        entering.set(None);
    }
}

/// A swipe lifted from a card that is no longer the top (the frame for an earlier undo landed
/// mid-swipe) keeps only an undo: any other swipe would act on a card the finger never touched,
/// so it is dropped.
fn keep_only_undo_off_top(mut state: Signal<SwipeState>, on_top: bool) {
    if !on_top && state.peek().latest_swipe != Some(Direction::Down) {
        state.write().latest_swipe = None;
    }
}

/// Dispatches the committed latest swipe from `state` for `card`. For an
/// exit direction, pushes the card onto the overlay and fires its callback
/// at once, so the stack advances on the next render; a down swipe fires
/// undo.
fn dispatch_latest(
    mut state: Signal<SwipeState>,
    mut exiting_overlay: Signal<Vec<ExitingCard>>,
    handlers: SwipeHandlers,
    card: &Card,
) {
    let Some(dir) = state.peek().latest_swipe else {
        return;
    };
    state.write().latest_swipe = None;
    let on_exit = match dir {
        Direction::Down => {
            handlers.down.call(card.clone());
            return;
        }
        Direction::Left => handlers.left,
        Direction::Right => handlers.right,
        Direction::Up => handlers.up,
    };
    // Seed the exit from the release position, mirroring the drag
    // transform's axis clamping.
    let delta = state
        .peek()
        .release_delta
        .unwrap_or(DeltaPoint::new(0.0, 0.0));
    let (from_x, from_y) = match dir {
        Direction::Up => (0.0, delta.y.min(0.0)),
        Direction::Left | Direction::Right | Direction::Down => (delta.x, 0.0),
    };
    exiting_overlay.write().push(ExitingCard {
        id: card.scryfall_data.id,
        direction: dir,
        card: card.clone(),
        from_x,
        from_y,
        from_rot: from_x * TILT_PER_PX,
    });
    on_exit.call(card.clone());
}

/// A peeking card stack with gesture-driven commit-or-return behavior.
#[component]
pub fn SwipeStack(
    /// Window of cards to render, top-first. Typically
    /// `cards[current_index..].take(STACK_DEPTH)`.
    cards: Vec<Card>,
    /// Gesture thresholds and allowed directions.
    config: SwipeConfig,
    /// Fires immediately on left-swipe commit (skip).
    on_swipe_left: EventHandler<Card>,
    /// Fires immediately on right-swipe commit (add/remove).
    on_swipe_right: EventHandler<Card>,
    /// Fires immediately on up-swipe commit (maybeboard).
    on_swipe_up: EventHandler<Card>,
    /// Fires immediately on down-swipe (undo; no exit animation).
    on_swipe_down: EventHandler<Card>,
    /// When `Some(dir)`, the new top card plays a keyframe entering from
    /// `dir`. The stack clears this signal on `animationend`.
    entering: Signal<Option<Direction>>,
    /// Fires once the top card's image has its bytes, failed, or has no art.
    #[props(default)]
    on_top_shown: Option<EventHandler<()>>,
    /// Whether to draw the ghost card under the cards until the top card's
    /// image is in and has had [`TOP_PAINT_GRACE`] to paint, while the cards
    /// under the top stay hidden, so neither an empty slot nor another card
    /// shows. The ghost is a stack card in the top card's own box.
    #[props(default)]
    ghost: bool,
    /// Keeps the ghost up regardless, for a pile the host has yet to show.
    #[props(default)]
    hold: bool,
    /// Every card's shown face where the screen keeps them: the top card's flip writes it, and
    /// a flip the screen writes elsewhere (its details drawer) turns the image.
    #[props(default)]
    faces: Option<Signal<HashMap<Uuid, usize>>>,
) -> Element {
    // The cards whose image has loaded and decoded, failed, or has no art,
    // and the top whose image came in while it was the top, while it paints.
    let mut shown_cards: Signal<HashSet<Uuid>> = use_signal(HashSet::new);
    let mut painting: Signal<Option<Uuid>> = use_signal(|| None);
    let mut state = use_signal(SwipeState::new);
    // In-flight exiting cards. Each entry is rendered as an overlay layer
    // playing a one-shot CSS keyframe and removes itself on animationend.
    // Multiple cards may be exiting concurrently during rapid swiping.
    let mut exiting_overlay: Signal<Vec<ExitingCard>> = use_signal(Vec::new);

    let visible = cards.into_iter().take(STACK_DEPTH).collect::<Vec<_>>();
    let n = visible.len();
    let top_id = visible.first().map(|c| c.scryfall_data.id);
    let held = hold || ghost_held(top_id, &shown_cards.read(), painting());
    // A top whose image came in while it was the top has the grace to paint
    // before the ghost goes and the cards under it may show.
    let painting_now = painting();
    use_effect(use_reactive!(|painting_now| {
        if let Some(id) = painting_now {
            spawn(async move {
                tokio::time::sleep(TOP_PAINT_GRACE).await;
                if *painting.peek() == Some(id) {
                    painting.set(None);
                }
            });
        }
    }));

    let handlers = SwipeHandlers {
        left: on_swipe_left,
        right: on_swipe_right,
        up: on_swipe_up,
        down: on_swipe_down,
    };
    let top_touch = visible.first().map(|c| (c.clone(), config.clone()));
    let top_mouse = top_touch.clone();
    let layers = visible.into_iter().enumerate().rev().map(|(i, card)| {
        let is_top = i == 0;
        let peek_i = gesture_math::layout_index(i);
        // i < n, so this never floors.
        let z = n.saturating_sub(i);

        // The peeking cards take the `.swipe-stack-card` class's
        // transition (0.12s ease-out), so a promotion to the top
        // interpolates. The top card sets its own: none while it is
        // dragged, an overshoot back to rest once it is let go short.
        let style = if is_top {
            // Top card, following the gesture (or sitting at rest).
            let s = state();
            let delta = s
                .delta_from_start_point()
                .unwrap_or(DeltaPoint::new(0.0, 0.0));
            let tx = if s.traversing_axis == Some(Axis::X) {
                delta.x
            } else {
                0.0
            };
            // Suppress visual movement on downward drags. The
            // gesture is still tracked (so set_latest_swipe will
            // fire `Direction::Down` on threshold); only the
            // rendered transform is clamped. Undo then triggers
            // and the previous card slides in via the existing
            // entering keyframe. Upward movement is likewise only
            // shown when `Up` is an exit direction; screens where up
            // does nothing (e.g. commander swipe) keep the card put.
            let ty = if s.traversing_axis == Some(Axis::Y) {
                if config.allowed_directions.contains(&Direction::Up) {
                    delta.y.min(0.0)
                } else {
                    0.0
                }
            } else {
                0.0
            };
            let rot = tx * TILT_PER_PX;
            // Dragging: no transition, follow the finger 1:1.
            // Released below threshold: return to center with an
            // overshoot curve (easeOutBack) so the card bobbles
            // just past rest and settles, instead of a flat glide.
            let transition_override = if s.is_swiping {
                "transition: transform 0s;"
            } else {
                "transition: transform 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);"
            };
            format!(
                "transform: translate({tx}px, {ty}px) rotate({rot}deg); \
                     {transition_override} \
                     z-index: {z};"
            )
        } else {
            // Peeking layer behind the top card.
            let ty = peek_i * PEEK_OFFSET_PX;
            let scale = 1.0 - peek_i * PEEK_SCALE_STEP;
            format!("transform: translateY({ty}px) scale({scale}); z-index: {z};")
        };

        // Apply an enter-keyframe class only on the top card, and
        // only when the parent requested one (after undo).
        let entering_class = if is_top {
            match entering() {
                Some(Direction::Left) => " card-stack-enter-left",
                Some(Direction::Right) => " card-stack-enter-right",
                Some(Direction::Up) => " card-stack-enter-up",
                Some(Direction::Down) => " card-stack-enter-down",
                None => "",
            }
        } else {
            ""
        };

        let class = format!(
            "swipe-stack-card{}{}",
            if is_top { " top" } else { "" },
            entering_class
        );

        let card_id = card.scryfall_data.id;
        Layer {
            id: card_id,
            name: card.scryfall_data.name.clone(),
            is_top,
            class,
            style,
            loaded: shown_cards.read().contains(&card_id),
            touch: top_touch.clone(),
            mouse: top_mouse.clone(),
            profile: card.card_profile,
            sd: card.scryfall_data,
        }
    });

    rsx! {
        div {
            class: if held { "swipe-stack ghost-held" } else { "swipe-stack" },
            // The cards under the top have no pointer, so a gesture that
            // misses the top card while it comes back from an undo lands
            // here. It ends the return ([`take_hold`]), the card at rest, and
            // acts on the top card, so no swipe made during a return is lost.
            ontouchstart: move |e: Event<TouchData>| {
                if entering.peek().is_none() { return; }
                take_hold(&mut entering);
                state.ontouchstart(e);
            },
            ontouchmove: move |e: Event<TouchData>| {
                if state.peek().is_swiping { state.ontouchmove(e); }
            },
            ontouchend: move |e: Event<TouchData>| {
                let Some((card, config)) = &top_touch else { return; };
                if !state.peek().is_swiping { return; }
                state.ontouchend(e, config);
                dispatch_latest(state, exiting_overlay, handlers, card);
            },
            // A touch the browser cancels ends with no swipe and leaves
            // nothing behind for the next one.
            ontouchcancel: move |_| state.write().reset(),
            onmousedown: move |e: Event<MouseData>| {
                if entering.peek().is_none() { return; }
                take_hold(&mut entering);
                state.onmousedown(e);
            },
            onmousemove: move |e: Event<MouseData>| {
                if state.peek().is_swiping { state.onmousemove(e); }
            },
            onmouseup: move |e: Event<MouseData>| {
                let Some((card, config)) = &top_mouse else { return; };
                if !state.peek().is_swiping { return; }
                state.onmouseup(e, config);
                dispatch_latest(state, exiting_overlay, handlers, card);
            },
            if held && ghost {
                GhostCard {}
            }
            // Render back-to-front so the DOM order matches z-index painting.
            // Keyed by card, the loop's own node, so a swipe drops the top and
            // keeps every other card mounted.
            for layer in layers {
                div {
                    key: "{layer.id}",
                    class: "{layer.class}",
                    style: "{layer.style}",
                    "aria-label": "{layer.name}",

                    // A gesture starts on the top card, and the stack under it
                    // never sees its events. The cards under the top have no
                    // pointer, so one under the top takes a touch only while
                    // the page still draws it on top after the app moved it
                    // down (an undo's frame not yet painted): that touch starts
                    // the gesture too, and only an undo carries from it.
                    // One that starts while the card comes back from an undo
                    // ends the return first ([`take_hold`]), so the keyframe and
                    // the drag never fight. Once it is under way it is the
                    // stack's: the frame for an earlier undo can land mid-swipe
                    // and take this card off the top, and the finger's moves and
                    // lift still reach it and act on the top card.
                    ontouchstart: move |e: Event<TouchData>| {
                        e.stop_propagation();
                        take_hold(&mut entering);
                        state.ontouchstart(e);
                    },
                    ontouchmove: move |e: Event<TouchData>| {
                        e.stop_propagation();
                        if !state.peek().is_swiping { return; }
                        state.ontouchmove(e);
                    },
                    ontouchend: move |e: Event<TouchData>| {
                        e.stop_propagation();
                        if !state.peek().is_swiping { return; }
                        let Some((card, config)) = &layer.touch else { return; };
                        state.ontouchend(e, config);
                        keep_only_undo_off_top(state, layer.is_top);
                        dispatch_latest(state, exiting_overlay, handlers, card);
                    },

                    ontouchcancel: move |e: Event<TouchData>| {
                        e.stop_propagation();
                        state.write().reset();
                    },

                    onmousedown: move |e: Event<MouseData>| {
                        e.stop_propagation();
                        take_hold(&mut entering);
                        state.onmousedown(e);
                    },
                    onmousemove: move |e: Event<MouseData>| {
                        e.stop_propagation();
                        if !state.peek().is_swiping { return; }
                        state.onmousemove(e);
                    },
                    onmouseup: move |e: Event<MouseData>| {
                        e.stop_propagation();
                        if !state.peek().is_swiping { return; }
                        let Some((card, config)) = &layer.mouse else { return; };
                        state.onmouseup(e, config);
                        keep_only_undo_off_top(state, layer.is_top);
                        dispatch_latest(state, exiting_overlay, handlers, card);
                    },

                    onanimationend: move |_| {
                        // Enter keyframe finished: clear the request.
                        if layer.is_top && entering.peek().is_some() {
                            entering.set(None);
                        }
                    },

                    StackCardImage {
                        sd: layer.sd,
                        profile: layer.profile,
                        is_top: layer.is_top,
                        loaded: layer.loaded,
                        faces,
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

            // In-flight exiting cards. Each plays a one-shot CSS keyframe and
            // removes itself from the overlay on animationend. Down-swipes
            // never enter the overlay (they trigger undo synchronously), so
            // we filter them defensively.
            for exiting in exiting_overlay().into_iter().filter(|e| !matches!(e.direction, Direction::Down)) {
                {
                    let exit_class = match exiting.direction {
                        Direction::Left => "card-stack-exit-left",
                        Direction::Right => "card-stack-exit-right",
                        Direction::Up => "card-stack-exit-up",
                        Direction::Down => "",
                    };
                    let id = exiting.id;
                    // The keyframes read these custom properties for their
                    // `from` frame, the card departs from where it was
                    // released instead of clipping back to rest.
                    let style = format!(
                        "z-index: 100; --exit-from-x: {}px; --exit-from-y: {}px; --exit-from-rot: {}deg;",
                        exiting.from_x, exiting.from_y, exiting.from_rot
                    );
                    rsx! {
                        div {
                            key: "exit-{id}",
                            class: "swipe-stack-card swipe-stack-exiting {exit_class}",
                            style: "{style}",
                            onanimationend: move |_| {
                                exiting_overlay.write().retain(|e| e.id != id);
                            },

                            // Same element as the resting/peek cards (non-flippable
                            // here) so the fly-off matches, and image-less cards
                            // show their text frame instead of going blank.
                            FlippableCardImage {
                                profile: Some(exiting.card.card_profile),
                                sd: exiting.card.scryfall_data,
                                size: ImageSize::Large,
                                class: "card-image".to_string(),
                                draggable: false,
                                flippable: false,
                                // On the face it showed as it went.
                                initial_face: faces.map_or(0, |faces| faces.peek().get(&id).copied().unwrap_or(0)),
                            }
                        }
                    }
                }
            }
        }
    }
}

/// One stack card's image, the top one flippable, on its face in `faces` when the screen shares
/// them. It shows at once if this stack already had it in (`loaded`), else only from its own load.
#[component]
fn StackCardImage(
    sd: ScryfallData,
    profile: CardProfile,
    is_top: bool,
    loaded: bool,
    faces: Option<Signal<HashMap<Uuid, usize>>>,
    on_load: EventHandler<()>,
) -> Element {
    let id = sd.id;
    let face = use_shared_face(faces, id);
    rsx! {
        FlippableCardImage {
            sd,
            profile: Some(profile),
            size: ImageSize::Large,
            class: "card-image".to_string(),
            draggable: false,
            flippable: is_top,
            trust_seen: loaded,
            face,
            on_face_change: share_face(faces, id),
            on_load,
        }
    }
}

#[cfg(test)]
mod tests {
    use dioxus::dioxus_core::NoOpMutations;

    use super::*;

    #[test]
    fn the_app_s_stack_keeps_every_drag_from_the_browser() {
        // A vertical drag on the stack is a swipe (down undoes, up saves); offered to the
        // browser as a pan, Android can take it and cancel the touch, losing the swipe.
        const MAIN: &str = include_str!("../../../../../../assets/main.css");
        let (_, rule) = MAIN
            .split_once("\n.swipe-stack {")
            .expect("the app's own rule for the stack");
        let (body, _) = rule.split_once('}').expect("its end");
        assert!(body.contains("touch-action: none;"), "{body}");
    }

    #[test]
    fn a_grab_during_a_return_ends_the_return_so_the_gesture_counts() {
        fn app() -> Element {
            let mut entering = use_signal(|| Some(Direction::Left));
            use_hook(move || take_hold(&mut entering));
            rsx! { p { "{entering():?}" } }
        }
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dom.render_immediate(&mut NoOpMutations);
        let html = dioxus_ssr::render(&dom);
        assert!(
            html.contains("<p>None</p>"),
            "the card snaps to rest: {html}"
        );
    }

    /// A real card under its own id and name.
    fn card(n: u128) -> Card {
        let mut card: Card = serde_json::from_str(include_str!(
            "../../../../../../../zwipe-ui/src/test_support/card.json"
        ))
        .unwrap();
        card.scryfall_data.id = Uuid::from_u128(n);
        card.scryfall_data.name = format!("Card {n}");
        card
    }

    /// One touch at `(x, y)`, as a touch event's data.
    fn touch_at(x: f64, y: f64, ended: bool) -> std::rc::Rc<dyn std::any::Any> {
        let point = serde_json::json!({
            "identifier": 0, "client_x": x, "client_y": y, "page_x": x, "page_y": y,
            "screen_x": x, "screen_y": y, "force": 1.0, "radius_x": 1.0, "radius_y": 1.0,
            "rotation_angle": 0.0
        });
        let touches = if ended {
            serde_json::json!([])
        } else {
            serde_json::json!([point])
        };
        let data: dioxus::html::SerializedTouchData = serde_json::from_value(serde_json::json!({
            "alt_key": false, "ctrl_key": false, "meta_key": false, "shift_key": false,
            "touches": touches, "changed_touches": [point], "target_touches": touches
        }))
        .unwrap();
        std::rc::Rc::new(dioxus::html::PlatformEventData::new(Box::new(data)))
    }

    /// The host of a burst test: five cards, the cursor on the fifth, and the newest swipe's
    /// card coming back from the left, as an Undo in flight leaves it. Each down swipe undoes
    /// one more step.
    #[derive(Clone, Copy)]
    struct Host {
        index: Signal<usize>,
        undos: Signal<usize>,
        rights: Signal<usize>,
    }

    fn burst_app() -> Element {
        let cards: Vec<Card> = (0..5).map(card).collect();
        let mut index = use_signal(|| 3_usize);
        let mut undos = use_signal(|| 0_usize);
        let mut rights = use_signal(|| 0_usize);
        let mut entering = use_signal(|| Some(Direction::Left));
        use_context_provider(|| Host {
            index,
            undos,
            rights,
        });
        let window: Vec<Card> = cards.get(index()..).map_or_else(Vec::new, <[Card]>::to_vec);
        let config = SwipeConfig::new(
            vec![
                Direction::Left,
                Direction::Right,
                Direction::Up,
                Direction::Down,
            ],
            90.0,
            2.5,
        );
        let none = move |_: Card| {};
        rsx! {
            SwipeStack {
                cards: window,
                config,
                on_swipe_left: none,
                on_swipe_right: move |_: Card| *rights.write() += 1,
                on_swipe_up: none,
                on_swipe_down: move |_: Card| {
                    let back = index.peek().saturating_sub(1);
                    index.set(back);
                    entering.set(Some(Direction::Left));
                    *undos.write() += 1;
                },
                entering,
            }
        }
    }

    /// Element ids by role: the stack (the first element taking touches) and each card by name.
    fn ids(
        edits: &[dioxus::dioxus_core::Mutation],
        stack: &mut Option<dioxus::dioxus_core::ElementId>,
        cards: &mut HashMap<String, dioxus::dioxus_core::ElementId>,
    ) {
        for edit in edits {
            if let dioxus::dioxus_core::Mutation::NewEventListener { name, id } = edit
                && name == "touchstart"
                && stack.is_none()
            {
                *stack = Some(*id);
            }
            if let dioxus::dioxus_core::Mutation::SetAttribute {
                name,
                value: dioxus::dioxus_core::AttributeValue::Text(v),
                id,
                ..
            } = edit
                && *name == "aria-label"
            {
                cards.insert(v.clone(), *id);
            }
        }
    }

    /// Three quick down swipes during one return, each on `target` (the stack, or the top card).
    fn burst(on_top: bool) -> (usize, usize, String) {
        dioxus::html::set_event_converter(Box::new(dioxus::html::SerializedHtmlEventConverter));
        let mut dom = VirtualDom::new(burst_app);
        let mut edits = dioxus::dioxus_core::Mutations::default();
        dom.rebuild(&mut edits);
        let (mut stack, mut cards) = (None, HashMap::new());
        ids(&edits.edits, &mut stack, &mut cards);
        let host = dom.in_scope(ScopeId::APP, consume_context::<Host>);
        for _ in 0..3 {
            let top = format!("Card {}", *host.index.peek());
            let target = if on_top {
                cards.get(&top).copied()
            } else {
                stack
            }
            .unwrap();
            let (x, mut y) = (200.0, 300.0);
            dom.runtime().handle_event(
                "touchstart",
                Event::new(touch_at(x, y, false), true),
                target,
            );
            for _ in 0..4 {
                y += 30.0;
                dom.runtime().handle_event(
                    "touchmove",
                    Event::new(touch_at(x, y, false), true),
                    target,
                );
            }
            dom.runtime()
                .handle_event("touchend", Event::new(touch_at(x, y, true), true), target);
            let mut more = dioxus::dioxus_core::Mutations::default();
            dom.render_immediate(&mut more);
            ids(&more.edits, &mut stack, &mut cards);
        }
        let html = dioxus_ssr::render(&dom);
        let entering = html.matches("card-stack-enter-").count();
        (*host.undos.peek(), entering, html)
    }

    #[test]
    fn three_down_swipes_during_one_return_undo_three_steps_on_the_stack() {
        let (undos, entering, html) = burst(false);
        assert_eq!(undos, 3, "{html}");
        assert_eq!(entering, 1, "only the last card comes in: {html}");
    }

    #[test]
    fn three_down_swipes_during_one_return_undo_three_steps_on_the_top_card() {
        let (undos, entering, html) = burst(true);
        assert_eq!(undos, 3, "{html}");
        assert_eq!(entering, 1, "only the last card comes in: {html}");
    }

    /// Three quick down swipes as Android delivers them: each starts on the card that was on
    /// top in the last frame drawn, and the frame for the undo before it lands mid-swipe, so
    /// the card under the finger stops being the top before the finger lifts.
    #[test]
    fn a_swipe_that_outlives_its_card_s_turn_on_top_still_undoes() {
        dioxus::html::set_event_converter(Box::new(dioxus::html::SerializedHtmlEventConverter));
        let mut dom = VirtualDom::new(burst_app);
        let mut edits = dioxus::dioxus_core::Mutations::default();
        dom.rebuild(&mut edits);
        let (mut stack, mut cards) = (None, HashMap::new());
        ids(&edits.edits, &mut stack, &mut cards);
        let host = dom.in_scope(ScopeId::APP, consume_context::<Host>);
        let mut drawn_top = *host.index.peek();
        for _ in 0..3 {
            let target = cards.get(&format!("Card {drawn_top}")).copied().unwrap();
            let (x, mut y) = (200.0, 300.0);
            dom.runtime().handle_event(
                "touchstart",
                Event::new(touch_at(x, y, false), true),
                target,
            );
            // The last undo's frame lands while the finger is down.
            let mut more = dioxus::dioxus_core::Mutations::default();
            dom.render_immediate(&mut more);
            ids(&more.edits, &mut stack, &mut cards);
            drawn_top = *host.index.peek();
            for _ in 0..4 {
                y += 30.0;
                dom.runtime().handle_event(
                    "touchmove",
                    Event::new(touch_at(x, y, false), true),
                    target,
                );
            }
            dom.runtime()
                .handle_event("touchend", Event::new(touch_at(x, y, true), true), target);
        }
        let mut more = dioxus::dioxus_core::Mutations::default();
        dom.render_immediate(&mut more);
        let html = dioxus_ssr::render(&dom);
        assert_eq!(*host.undos.peek(), 3, "{html}");
        assert_eq!(
            html.matches("card-stack-enter-").count(),
            1,
            "only the last card comes in: {html}"
        );
    }

    /// A swipe right that starts on the top card of the last frame, whose undo frame then lands
    /// mid-swipe: the card under the finger is no longer the top, so the swipe adds nothing.
    #[test]
    fn a_side_swipe_that_outlives_its_card_s_turn_on_top_does_nothing() {
        dioxus::html::set_event_converter(Box::new(dioxus::html::SerializedHtmlEventConverter));
        let mut dom = VirtualDom::new(burst_app);
        let mut edits = dioxus::dioxus_core::Mutations::default();
        dom.rebuild(&mut edits);
        let (mut stack, mut cards) = (None, HashMap::new());
        ids(&edits.edits, &mut stack, &mut cards);
        let host = dom.in_scope(ScopeId::APP, consume_context::<Host>);
        let drawn_top = cards.get("Card 3").copied().unwrap();
        let swipe = |dom: &mut VirtualDom,
                     target,
                     dx: f64,
                     dy: f64,
                     mid: Option<&mut dioxus::dioxus_core::Mutations>| {
            let (mut x, mut y) = (200.0, 300.0);
            dom.runtime().handle_event(
                "touchstart",
                Event::new(touch_at(x, y, false), true),
                target,
            );
            if let Some(m) = mid {
                dom.render_immediate(m);
            }
            for _ in 0..4 {
                x += dx;
                y += dy;
                dom.runtime().handle_event(
                    "touchmove",
                    Event::new(touch_at(x, y, false), true),
                    target,
                );
            }
            dom.runtime()
                .handle_event("touchend", Event::new(touch_at(x, y, true), true), target);
        };
        // A down swipe undoes; its frame is not drawn yet.
        swipe(&mut dom, drawn_top, 0.0, 30.0, None);
        assert_eq!(*host.undos.peek(), 1);
        // A swipe right on the card still drawn on top; the undo's frame lands mid-swipe.
        let mut more = dioxus::dioxus_core::Mutations::default();
        swipe(&mut dom, drawn_top, 40.0, 0.0, Some(&mut more));
        assert_eq!(
            *host.rights.peek(),
            0,
            "nothing added for a card the finger never touched"
        );
    }

    /// Three quick down swipes where the page lags the app: each lands on the card that was the
    /// top before the last undo, which the app has already drawn as a card under the top while
    /// the page still shows it on top.
    #[test]
    fn a_swipe_that_lands_on_a_card_the_app_already_moved_under_the_top_still_undoes() {
        dioxus::html::set_event_converter(Box::new(dioxus::html::SerializedHtmlEventConverter));
        let mut dom = VirtualDom::new(burst_app);
        let mut edits = dioxus::dioxus_core::Mutations::default();
        dom.rebuild(&mut edits);
        let (mut stack, mut cards) = (None, HashMap::new());
        ids(&edits.edits, &mut stack, &mut cards);
        let host = dom.in_scope(ScopeId::APP, consume_context::<Host>);
        let mut shown_top = *host.index.peek();
        for _ in 0..3 {
            let target = cards.get(&format!("Card {shown_top}")).copied().unwrap();
            let (x, mut y) = (200.0, 300.0);
            dom.runtime().handle_event(
                "touchstart",
                Event::new(touch_at(x, y, false), true),
                target,
            );
            for _ in 0..4 {
                y += 30.0;
                dom.runtime().handle_event(
                    "touchmove",
                    Event::new(touch_at(x, y, false), true),
                    target,
                );
            }
            dom.runtime()
                .handle_event("touchend", Event::new(touch_at(x, y, true), true), target);
            // The app draws the undo before the next touch; the page shows the old top a little longer.
            let mut more = dioxus::dioxus_core::Mutations::default();
            dom.render_immediate(&mut more);
            ids(&more.edits, &mut stack, &mut cards);
            shown_top = (*host.index.peek()).saturating_add(1);
        }
        let mut more = dioxus::dioxus_core::Mutations::default();
        dom.render_immediate(&mut more);
        let html = dioxus_ssr::render(&dom);
        assert_eq!(*host.undos.peek(), 3, "{html}");
        assert_eq!(
            html.matches("card-stack-enter-").count(),
            1,
            "only the last card comes in: {html}"
        );
    }
}
