//! Stacked card swipe component.
//!
//! Renders up to [`STACK_DEPTH`] cards from a slice as a peeking stack. Only
//! the top card tracks gestures. When the top card commits a swipe it exits
//! off-screen (no rebound) via a CSS transform transition, then the parent's
//! callback advances state and the card beneath becomes the new top.
//!
//! Undo uses the `entering` signal: when set to `Some(dir)`, the freshly-
//! restored top card plays a keyframe animation entering from `dir`, then the
//! stack clears the signal on `animationend`.

use crate::inbound::components::interactions::swipe::{
    axis::Axis, config::SwipeConfig, direction::Direction, onmouse::OnMouse, ontouch::OnTouch,
    state::SwipeState,
};
use dioxus::html::geometry::euclid::{Point2D, UnknownUnit};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::image_uris::ImageUris;

type DeltaPoint = Point2D<f64, UnknownUnit>;

/// How many cards are rendered at once (top + peeking underneath).
pub const STACK_DEPTH: usize = 10;

/// Vertical offset per peek layer in pixels.
const PEEK_OFFSET_PX: f64 = 6.0;
/// Scale reduction per peek layer.
const PEEK_SCALE_STEP: f64 = 0.03;
/// Degrees of rotation applied per pixel of horizontal drag (the "tilt").
const TILT_PER_PX: f64 = 0.06;
/// Far off-screen X distance used for horizontal exits.
const EXIT_DISTANCE_X: f64 = 1200.0;
/// Far off-screen Y distance used for vertical exits.
const EXIT_DISTANCE_Y: f64 = 1400.0;
/// Rotation applied to horizontal exit transforms (matches the tilt feel).
const EXIT_TILT_DEG: f64 = 25.0;

/// Computes the off-screen destination `(x, y, rotation_deg)` for an exit.
fn exit_transform(direction: &Direction) -> (f64, f64, f64) {
    match direction {
        Direction::Left => (-EXIT_DISTANCE_X, 0.0, -EXIT_TILT_DEG),
        Direction::Right => (EXIT_DISTANCE_X, 0.0, EXIT_TILT_DEG),
        Direction::Up => (0.0, -EXIT_DISTANCE_Y, 0.0),
        Direction::Down => (0.0, EXIT_DISTANCE_Y, 0.0),
    }
}

/// A peeking card stack with gesture-driven commit-or-return behavior.
#[component]
pub fn SwipeStack(
    /// Window of cards to render, top-first. Typically
    /// `cards[current_index..].take(STACK_DEPTH)`.
    cards: Vec<Card>,
    /// Gesture thresholds and allowed directions.
    config: SwipeConfig,
    /// Fires once the left-swipe exit animation completes.
    on_swipe_left: EventHandler<Card>,
    /// Fires once the right-swipe exit animation completes.
    on_swipe_right: EventHandler<Card>,
    /// Fires once the up-swipe exit animation completes.
    on_swipe_up: EventHandler<Card>,
    /// Fires immediately on down-swipe (undo; no exit animation).
    on_swipe_down: EventHandler<Card>,
    /// When `Some(dir)`, the new top card plays a keyframe entering from
    /// `dir`. The stack clears this signal on `animationend`.
    entering: Signal<Option<Direction>>,
) -> Element {
    let mut state = use_signal(SwipeState::new);
    // Tracks (card_id, direction) of the card currently exiting off-screen.
    // Stale values are harmless — style is only applied when ids match.
    let exit_target: Signal<Option<(Uuid, Direction)>> = use_signal(|| None);

    let visible = cards.into_iter().take(STACK_DEPTH).collect::<Vec<_>>();
    let n = visible.len();

    // If the front card is currently exiting and there's a card behind it,
    // promote that card to the effective top **immediately** — both visually
    // (it transitions to origin in parallel with the front's exit) and for
    // gesture handling (its touch listeners stop bailing). This eliminates
    // the ~700ms perceived delay between swiping the top card and the next
    // one becoming swipeable, because A's exit and B's promotion now run
    // concurrently over the same 0.2s transition instead of sequentially.
    let front_is_exiting = match (visible.first(), exit_target().as_ref()) {
        (Some(front), Some((exit_id, _))) => *exit_id == front.scryfall_data.id,
        _ => false,
    };
    let effective_top_index: usize = if front_is_exiting && visible.len() > 1 { 1 } else { 0 };

    // Container-level transitionend handler. Any transform transition that
    // completes inside the stack bubbles up here. We process the pending exit
    // exactly once: read `exit_target`, find the matching card, clear the
    // signal, and fire the parent callback. Multiple bubbling events from
    // sibling transitions (e.g. the new top's peek→origin grow) hit the same
    // handler but find `exit_target = None` and no-op.
    let stack_visible_for_handler = visible.clone();
    let stack_transitionend = {
        let mut exit_target = exit_target;
        let on_swipe_left = on_swipe_left;
        let on_swipe_right = on_swipe_right;
        let on_swipe_up = on_swipe_up;
        move |_e: Event<TransitionData>| {
            let Some((id, dir)) = exit_target.peek().clone() else { return; };
            let Some(card) = stack_visible_for_handler
                .iter()
                .find(|c| c.scryfall_data.id == id)
                .cloned()
            else {
                // Exit-target points to a card not in this slice — clear it
                // so future swipes aren't blocked.
                exit_target.set(None);
                return;
            };
            // Clear BEFORE the parent callback so the next render sees both
            // `exit_target = None` and `current_index` advanced in the same
            // batch. The exiting card's key leaves the slice and its DOM
            // element is removed before any paint, preventing snap-back.
            exit_target.set(None);
            match dir {
                Direction::Left => on_swipe_left.call(card),
                Direction::Right => on_swipe_right.call(card),
                Direction::Up => on_swipe_up.call(card),
                Direction::Down => {}
            }
        }
    };

    rsx! {
        div {
            class: "swipe-stack",
            ontransitionend: stack_transitionend,
            // Render back-to-front so the DOM order matches z-index painting.
            for (i, card) in visible.into_iter().enumerate().rev() {
                {
                    let is_top = i == effective_top_index;
                    let is_exiting = i < effective_top_index;
                    let card_id = card.scryfall_data.id;
                    let card_name = card.scryfall_data.name.clone();

                    // Peek depth is measured from the effective top (not the
                    // raw slice index) so the card immediately behind the top
                    // sits at depth 1 regardless of whether a sibling is
                    // currently exiting above it.
                    let peek_depth = (i.saturating_sub(effective_top_index)) as f64;
                    let z = (n - i) as i32;

                    let exit_for_this = if is_exiting {
                        exit_target().map(|(_, dir)| dir)
                    } else {
                        None
                    };

                    // Build the inline transform for this card. We let the
                    // `.swipe-stack-card` CSS class own the default transform
                    // transition (0.35s ease-out) so every transform change —
                    // peek→top promotion, return-to-origin, exit — interpolates
                    // smoothly without per-card bookkeeping. The ONLY case we
                    // override with `transition: transform 0s` is during an
                    // active drag, where the card must follow the finger
                    // instantly.
                    let style = if let Some(dir) = exit_for_this.as_ref() {
                        // Committed swipe — class transition carries it off-screen.
                        let (tx, ty, rot) = exit_transform(dir);
                        format!(
                            "transform: translate({tx}px, {ty}px) rotate({rot}deg); \
                             z-index: {z};"
                        )
                    } else if !is_top {
                        // Peeking layer behind the effective top card.
                        let ty = peek_depth * PEEK_OFFSET_PX;
                        let scale = 1.0 - peek_depth * PEEK_SCALE_STEP;
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
                        let ty = if s.traversing_axis == Some(Axis::Y) {
                            delta.y
                        } else {
                            0.0
                        };
                        let rot = tx * TILT_PER_PX;
                        // Disable transition mid-drag so the card follows the
                        // finger 1:1; otherwise let the class transition run.
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

                    // Apply an enter-keyframe class only on the effective top
                    // card, and only when the parent requested one.
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
                    let config_touch_start = config.clone();
                    let _ = config_touch_start; // start doesn't need config
                    let config_touch_end = config.clone();
                    let config_mouse_end = config.clone();
                    let card_for_handlers = card.clone();

                    // Dispatches the committed latest_swipe from SwipeState.
                    // Called from both touchend and mouseup branches.
                    let dispatch_latest = {
                        let card = card_for_handlers;
                        let mut exit_target = exit_target;
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
                                        // Exit transform takes over on next render.
                                        exit_target.set(Some((card.scryfall_data.id, d)));
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

                            // Guard: only the effective top can start gestures,
                            // and no new gestures may start while ANY card is
                            // mid-exit (committing during an exit would corrupt
                            // `exit_target`). The effective top is already
                            // promoted visually during the previous card's
                            // exit, so the perceived delay is just the exit
                            // transition duration.
                            ontouchstart: move |e: Event<TouchData>| {
                                if !is_top
                                    || exit_target.peek().is_some()
                                    || entering.peek().is_some() { return; }
                                state.ontouchstart(e);
                            },
                            ontouchmove: move |e: Event<TouchData>| {
                                if !is_top
                                    || exit_target.peek().is_some()
                                    || entering.peek().is_some() { return; }
                                state.ontouchmove(e);
                            },
                            ontouchend: {
                                let config = config_touch_end;
                                let mut dispatch = dispatch_latest.clone();
                                move |e: Event<TouchData>| {
                                    if !is_top
                                        || exit_target.peek().is_some()
                                        || entering.peek().is_some() { return; }
                                    state.ontouchend(e, &config);
                                    dispatch();
                                }
                            },

                            onmousedown: move |e: Event<MouseData>| {
                                if !is_top
                                    || exit_target.peek().is_some()
                                    || entering.peek().is_some() { return; }
                                state.onmousedown(e);
                            },
                            onmousemove: move |e: Event<MouseData>| {
                                if !is_top
                                    || exit_target.peek().is_some()
                                    || entering.peek().is_some() { return; }
                                state.onmousemove(e);
                            },
                            onmouseup: {
                                let config = config_mouse_end;
                                let mut dispatch = dispatch_latest;
                                move |e: Event<MouseData>| {
                                    if !is_top
                                        || exit_target.peek().is_some()
                                        || entering.peek().is_some() { return; }
                                    state.onmouseup(e, &config);
                                    dispatch();
                                }
                            },

                            // Exit transitions are handled at the stack
                            // container level via `ontransitionend` (see
                            // `stack_transitionend`). Per-card transitionend
                            // bindings caused rebinding races where the
                            // exiting card's handler captured stale
                            // `is_exiting` values.

                            onanimationend: move |_| {
                                // Enter keyframe finished — clear the request.
                                if is_top && entering.peek().is_some() {
                                    entering.set(None);
                                }
                            },

                            if let Some(ImageUris { large: Some(image_url), .. }) = &card.scryfall_data.image_uris {
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
