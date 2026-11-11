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
//! stack clears the signal on `animationend`.

use std::collections::HashSet;

use dioxus::html::geometry::euclid::{Point2D, UnknownUnit};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
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

/// A blank in a card image's shape (Scryfall's 672 by 936), which the ghost
/// card's image shows so it sizes as the card's image does.
const CARD_GHOST_SRC: &str =
    "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='672' height='936'/%3E";

/// The ghost of the top card: a stack card holding a card-shaped blank in
/// the card's own wrapper, so the card's own rules give it the top card's
/// box. Drawn in a `.swipe-stack`.
#[component]
pub(crate) fn GhostCard() -> Element {
    rsx! {
        div { class: "swipe-stack-card swipe-stack-ghost-card", "aria-hidden": "true",
            div { class: "flippable-card-wrapper card-image",
                div { class: "flip-face",
                    img { class: "img-loaded card-ghost", src: CARD_GHOST_SRC, alt: "", draggable: false }
                }
            }
        }
    }
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
    card_touch: Card,
    card_mouse: Card,
    sd: ScryfallData,
    config_touch: SwipeConfig,
    config_mouse: SwipeConfig,
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
        _ => (delta.x, 0.0),
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
            card_touch: card.clone(),
            card_mouse: card.clone(),
            sd: card.scryfall_data,
            config_touch: config.clone(),
            config_mouse: config.clone(),
        }
    });

    rsx! {
        div { class: if held { "swipe-stack ghost-held" } else { "swipe-stack" },
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

                    // Guards: only the top card receives gestures, and
                    // never during an enter (undo) keyframe so the
                    // animation isn't fought by inline transform.
                    ontouchstart: move |e: Event<TouchData>| {
                        if !layer.is_top || entering.peek().is_some() { return; }
                        state.ontouchstart(e);
                    },
                    ontouchmove: move |e: Event<TouchData>| {
                        if !layer.is_top || entering.peek().is_some() { return; }
                        state.ontouchmove(e);
                    },
                    ontouchend: move |e: Event<TouchData>| {
                        if !layer.is_top || entering.peek().is_some() { return; }
                        state.ontouchend(e, &layer.config_touch);
                        dispatch_latest(state, exiting_overlay, handlers, &layer.card_touch);
                    },

                    onmousedown: move |e: Event<MouseData>| {
                        if !layer.is_top || entering.peek().is_some() { return; }
                        state.onmousedown(e);
                    },
                    onmousemove: move |e: Event<MouseData>| {
                        if !layer.is_top || entering.peek().is_some() { return; }
                        state.onmousemove(e);
                    },
                    onmouseup: move |e: Event<MouseData>| {
                        if !layer.is_top || entering.peek().is_some() { return; }
                        state.onmouseup(e, &layer.config_mouse);
                        dispatch_latest(state, exiting_overlay, handlers, &layer.card_mouse);
                    },

                    onanimationend: move |_| {
                        // Enter keyframe finished: clear the request.
                        if layer.is_top && entering.peek().is_some() {
                            entering.set(None);
                        }
                    },

                    FlippableCardImage {
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
                                sd: exiting.card.scryfall_data,
                                size: ImageSize::Large,
                                class: "card-image".to_string(),
                                draggable: false,
                                flippable: false,
                            }
                        }
                    }
                }
            }
        }
    }
}
