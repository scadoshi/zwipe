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
use crate::inbound::components::interactions::swipe::{
    config::SwipeConfig, onmouse::OnMouse, ontouch::OnTouch, state::SwipeState,
};
use dioxus::html::geometry::euclid::{Point2D, UnknownUnit};
use dioxus::prelude::*;

type DeltaPoint = Point2D<f64, UnknownUnit>;

pub const VH_GAP: i32 = 75;
pub const VW_GAP: i32 = 100;

#[component]
pub fn Swipeable(state: Signal<SwipeState>, config: SwipeConfig, children: Element) -> Element {
    // cloning to be passable to closures
    let config1 = config.clone();
    let config2 = config.clone();

    // re-positioning during swipe reactive to state changes
    let delta = state()
        .delta_from_start_point()
        .unwrap_or(DeltaPoint::new(0.0, 0.0));

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

    // screens starting position relative to main screen - reactive to screen_offset
    let xvw = (config.from_main_screen.x + state().screen_offset.x) * VW_GAP;
    let yvh = (config.from_main_screen.y + state().screen_offset.y) * VH_GAP;

    // for returning element back to start position smoothly
    let return_animation_seconds = state().return_animation_seconds;

    rsx! {
            div { class : "swipeable",
                    style : format!(
                    "transform: translate(calc({xpx}px + {xvw}vw), calc({ypx}px + {yvh}vh));
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
