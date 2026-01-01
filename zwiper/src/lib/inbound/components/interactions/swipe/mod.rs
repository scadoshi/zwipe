pub mod axis;
pub mod config;
pub mod direction;
pub mod onmouse;
pub mod onswipe;
pub mod ontouch;
pub mod screen_offset;
pub mod state;
pub mod time_point;

use crate::inbound::components::interactions::swipe::axis::Axis;
use crate::inbound::components::interactions::swipe::direction::Direction;
use crate::inbound::components::interactions::swipe::{
    config::SwipeConfig, onmouse::OnMouse, ontouch::OnTouch, state::SwipeState,
};
use dioxus::html::geometry::euclid::{Point2D, UnknownUnit};
use dioxus::prelude::*;

type DeltaPoint = Point2D<f64, UnknownUnit>;

#[component]
pub fn Swipeable(
    state: Signal<SwipeState>,
    config: SwipeConfig,
    on_swipe_left: EventHandler<()>,
    on_swipe_right: EventHandler<()>,
    on_swipe_up: EventHandler<()>,
    on_swipe_down: EventHandler<()>,
    children: Element,
) -> Element {
    // cloning to be passable to closures
    let config1 = config.clone();
    let config2 = config.clone();

    // re-positioning during swipe reactive to state changes
    let delta = state()
        .delta_from_start_point()
        .unwrap_or(DeltaPoint::new(0.0, 0.0));

    // only move on the axis the user is swiping on (axis locking)
    let xpx = if state().traversing_axis == Some(Axis::X) {
        delta.x
    } else {
        0.0
    };

    let ypx = if state().traversing_axis == Some(Axis::Y) {
        delta.y
    } else {
        0.0
    };

    // for returning element back to start position smoothly
    let return_animation_seconds = state().return_animation_seconds;

    // Call EventHandlers when swipe is detected and clear the swipe
    use_effect(move || {
        if let Some(direction) = state().latest_swipe {
            match direction {
                Direction::Left => on_swipe_left.call(()),
                Direction::Right => on_swipe_right.call(()),
                Direction::Up => on_swipe_up.call(()),
                Direction::Down => on_swipe_down.call(()),
            }
            // Clear the swipe after handling it
            state.write().latest_swipe = None;
        }
    });

    rsx! {
            div {
                    style : format!(
                    "transform: translate({xpx}px, {ypx}px);
                    transition: transform {return_animation_seconds}s;"
                    ),

                    ontouchstart : move |e: Event<TouchData>| state.ontouchstart(e),
                    ontouchmove : move |e: Event<TouchData>| state.ontouchmove(e),
                    ontouchend : move |e: Event<TouchData>| state.ontouchend(e, &config1),

                    onmousedown : move |e: Event<MouseData>| state.onmousedown(e),
                    onmousemove : move |e: Event<MouseData>| state.onmousemove(e),
                    onmouseup : move |e: Event<MouseData>| state.onmouseup(e, &config2),

                    { children }
        }
    }
}
