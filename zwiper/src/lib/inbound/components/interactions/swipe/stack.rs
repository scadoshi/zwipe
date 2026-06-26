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
//!    advances right away — the card beneath becomes the new top on the very
//!    next render and is interactive without delay.
//!
//! Down-swipes do not exit. They fire `on_swipe_down` for undo and the active
//! card is visually clamped (Y delta cannot go positive) so it never moves on
//! screen during the gesture.
//!
//! Undo uses the `entering` signal: when set to `Some(dir)`, the freshly-
//! restored top card plays a keyframe animation entering from `dir`, then the
//! stack clears the signal on `animationend`.

use crate::inbound::components::interactions::swipe::{
    axis::Axis, config::SwipeConfig, direction::Direction, onmouse::OnMouse, ontouch::OnTouch,
    state::SwipeState,
};
use crate::inbound::screens::deck::card::components::flippable_card_image::FlippableCardImage;
use dioxus::html::geometry::euclid::{Point2D, UnknownUnit};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::ImageSize;

type DeltaPoint = Point2D<f64, UnknownUnit>;

/// How many cards are rendered at once (top + peeking underneath).
pub const STACK_DEPTH: usize = 10;

/// Vertical offset per peek layer in pixels.
const PEEK_OFFSET_PX: f64 = 0.0;
/// Scale reduction per peek layer.
const PEEK_SCALE_STEP: f64 = 0.0;
/// Degrees of rotation applied per pixel of horizontal drag (the "tilt").
const TILT_PER_PX: f64 = 0.06;

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
) -> Element {
    let mut state = use_signal(SwipeState::new);
    // In-flight exiting cards. Each entry is rendered as an overlay layer
    // playing a one-shot CSS keyframe and removes itself on animationend.
    // Multiple cards may be exiting concurrently during rapid swiping.
    let mut exiting_overlay: Signal<Vec<(Uuid, Direction, Card)>> = use_signal(Vec::new);

    let visible = cards.into_iter().take(STACK_DEPTH).collect::<Vec<_>>();
    let n = visible.len();

    rsx! {
        div { class: "swipe-stack",
            // Render back-to-front so the DOM order matches z-index painting.
            for (i, card) in visible.into_iter().enumerate().rev() {
                {
                    let is_top = i == 0;
                    let card_id = card.scryfall_data.id;
                    let card_name = card.scryfall_data.name.clone();
                    let peek_i = i as f64;
                    let z = (n - i) as i32;

                    // Build the inline transform for this card. We let the
                    // `.swipe-stack-card` CSS class own the default transform
                    // transition (0.12s ease-out) so peek→top promotion and
                    // sub-threshold return-to-origin both interpolate without
                    // per-card bookkeeping. The ONLY case we override with
                    // `transition: transform 0s` is during an active drag,
                    // where the card must follow the finger 1:1.
                    let style = if !is_top {
                        // Peeking layer behind the top card.
                        let ty = peek_i * PEEK_OFFSET_PX;
                        let scale = 1.0 - peek_i * PEEK_SCALE_STEP;
                        format!(
                            "transform: translateY({ty}px) scale({scale}); z-index: {z};"
                        )
                    } else {
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
                        // fire `Direction::Down` on threshold) — only the
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
                        let transition_override = if s.is_swiping {
                            "transition: transform 0s;"
                        } else {
                            ""
                        };
                        format!(
                            "transform: translate({tx}px, {ty}px) rotate({rot}deg); \
                             {transition_override} \
                             z-index: {z};"
                        )
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

                    // Gesture handlers — only bound on the top card.
                    let config_touch_end = config.clone();
                    let config_mouse_end = config.clone();
                    let card_for_handlers = card.clone();

                    // Dispatches the committed latest_swipe from SwipeState.
                    // For exit directions, push the card onto the overlay AND
                    // fire the parent callback immediately so current_index
                    // advances synchronously — the new top is interactive on
                    // the next render with no gesture-block guard.
                    let dispatch_latest = {
                        let card = card_for_handlers;
                        let mut state = state;
                        let on_swipe_down = on_swipe_down;
                        move || {
                            let latest = state.peek().latest_swipe.clone();
                            if let Some(dir) = latest {
                                state.write().latest_swipe = None;
                                match dir {
                                    Direction::Down => {
                                        on_swipe_down.call(card.clone());
                                    }
                                    d @ (Direction::Left
                                    | Direction::Right
                                    | Direction::Up) => {
                                        exiting_overlay.write().push((
                                            card.scryfall_data.id,
                                            d.clone(),
                                            card.clone(),
                                        ));
                                        match d {
                                            Direction::Left => on_swipe_left.call(card.clone()),
                                            Direction::Right => on_swipe_right.call(card.clone()),
                                            Direction::Up => on_swipe_up.call(card.clone()),
                                            _ => unreachable!(),
                                        }
                                    }
                                }
                            }
                        }
                    };

                    rsx! {
                        div {
                            key: "{card_id}",
                            class: "{class}",
                            style: "{style}",
                            "aria-label": "{card_name}",

                            // Guards: only the top card receives gestures, and
                            // never during an enter (undo) keyframe so the
                            // animation isn't fought by inline transform.
                            ontouchstart: move |e: Event<TouchData>| {
                                if !is_top || entering.peek().is_some() { return; }
                                state.ontouchstart(e);
                            },
                            ontouchmove: move |e: Event<TouchData>| {
                                if !is_top || entering.peek().is_some() { return; }
                                state.ontouchmove(e);
                            },
                            ontouchend: {
                                let config = config_touch_end;
                                let mut dispatch = dispatch_latest.clone();
                                move |e: Event<TouchData>| {
                                    if !is_top || entering.peek().is_some() { return; }
                                    state.ontouchend(e, &config);
                                    dispatch();
                                }
                            },

                            onmousedown: move |e: Event<MouseData>| {
                                if !is_top || entering.peek().is_some() { return; }
                                state.onmousedown(e);
                            },
                            onmousemove: move |e: Event<MouseData>| {
                                if !is_top || entering.peek().is_some() { return; }
                                state.onmousemove(e);
                            },
                            onmouseup: {
                                let config = config_mouse_end;
                                let mut dispatch = dispatch_latest;
                                move |e: Event<MouseData>| {
                                    if !is_top || entering.peek().is_some() { return; }
                                    state.onmouseup(e, &config);
                                    dispatch();
                                }
                            },

                            onanimationend: move |_| {
                                // Enter keyframe finished — clear the request.
                                if is_top && entering.peek().is_some() {
                                    entering.set(None);
                                }
                            },

                            FlippableCardImage {
                                sd: card.scryfall_data,
                                size: ImageSize::Large,
                                class: "card-image".to_string(),
                                draggable: false,
                                flippable: is_top,
                            }
                        }
                    }
                }
            }

            // In-flight exiting cards. Each plays a one-shot CSS keyframe and
            // removes itself from the overlay on animationend. Down-swipes
            // never enter the overlay (they trigger undo synchronously), so
            // we filter them defensively.
            for (id, dir, card) in exiting_overlay().into_iter().filter(|(_, d, _)| !matches!(d, Direction::Down)) {
                {
                    let exit_class = match dir {
                        Direction::Left => "card-stack-exit-left",
                        Direction::Right => "card-stack-exit-right",
                        Direction::Up => "card-stack-exit-up",
                        Direction::Down => "",
                    };
                    let card_name = card.scryfall_data.name.clone();
                    let image_url: Option<String> = card.scryfall_data.primary_image_url(ImageSize::Large).map(str::to_owned);
                    rsx! {
                        div {
                            key: "exit-{id}",
                            class: "swipe-stack-card swipe-stack-exiting {exit_class}",
                            style: "z-index: 100;",
                            onanimationend: move |_| {
                                exiting_overlay.write().retain(|(eid, _, _)| *eid != id);
                            },

                            if let Some(ref image_url) = image_url {
                                img {
                                    src: "{image_url}",
                                    alt: "{card_name}",
                                    class: "card-image",
                                    draggable: false,
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
