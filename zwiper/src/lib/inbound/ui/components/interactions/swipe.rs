pub mod axis;
pub mod config;
pub mod direction;
pub mod onmouse;
pub mod ontouch;
pub mod state;
pub mod time_point;

use crate::inbound::ui::components::interactions::swipe::axis::Axis;
use crate::inbound::ui::components::interactions::swipe::direction::Direction as Dir;
use crate::inbound::ui::components::interactions::swipe::{
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
    let config3 = config.clone();
    let config4 = config.clone();

    // re-positioning during swipe - reactive to state changes
    let delta = state
        .read()
        .delta_from_start_point()
        .unwrap_or(DeltaPoint::new(0.0, 0.0));

    let xpx = if state.read().traversing_axis == Some(Axis::X) {
        delta.x
    } else {
        0.0
    };

    let ypx = if state.read().traversing_axis == Some(Axis::Y) {
        delta.y
    } else {
        0.0
    };

    // screens starting position relative to main screen - reactive to screen_displacement
    let from_main_x = match config.from_main_screen {
        Some(Dir::Left) => -1,
        Some(Dir::Right) => 1,
        _ => 0,
    };
    let xvw = (from_main_x + state.read().screen_displacement.x) * VW_GAP;

    let from_main_y = match config.from_main_screen {
        Some(Dir::Up) => -1,
        Some(Dir::Down) => 1,
        _ => 0,
    };
    let yvh = (from_main_y + state.read().screen_displacement.y) * VH_GAP;

    // for returning element back to start position smoothly
    let return_animation_seconds = state.read().return_animation_seconds;

    rsx! {
            div { class : "swipeable",
                    style : format!(
                    "transform: translate(calc({xpx}px + {xvw}vw), calc({ypx}px + {yvh}vh));
                    transition: transform {return_animation_seconds}s;"
                    ),

                    ontouchstart : move |e: Event<TouchData>| state.ontouchstart(e),
                    ontouchmove : move |e: Event<TouchData>| state.ontouchmove(e, &config1),
                    ontouchend : move |e: Event<TouchData>| state.ontouchend(e, &config2),

                    onmousedown : move |e: Event<MouseData>| state.onmousedown(e),
                    onmousemove : move |e: Event<MouseData>| state.onmousemove(e, &config3),
                    onmouseup : move |e: Event<MouseData>| state.onmouseup(e, &config4),

                    { children }
        }
    }
}
