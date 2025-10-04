use dioxus::prelude::*;
use zwipe::domain::ascii_logo;

use crate::{
    screens::auth::{login::Login, register::Register},
    swipe::{self, Direction as Dir, OnMouse, OnTouch},
};

#[component]
pub fn Home() -> Element {
    let ascii_logo = ascii_logo::logo();
    let mut swipe_state = use_signal(|| swipe::State::new());

    rsx! {
        Login {swipe_state}

        div { class : "swipe-able",

            style : format!(
                "transform: translateY({}px);
                transition: transform {}s;",
                swipe_state.read().dy().from_start,
                swipe_state.read().transition_seconds
            ),

            ontouchstart : move |e: Event<TouchData>| swipe_state.ontouchstart(e),
            ontouchmove : move |e: Event<TouchData>| swipe_state.ontouchmove(e),
            ontouchend : move |e: Event<TouchData>| {
                swipe_state.ontouchend(e, &[Dir::Up, Dir::Down]);
                println!("direction => {:?}", swipe_state.read().previous_swipe);
            },

            onmousedown : move |e: Event<MouseData>| swipe_state.onmousedown(e),
            onmousemove : move |e: Event<MouseData>| swipe_state.onmousemove(e),
            onmouseup : move |e: Event<MouseData>| swipe_state.onmouseup(e, &[Dir::Up, Dir::Down]),

            div { class : "home-screen",

                div {
                    class : "home-direction-arrow",
                    "↑"
                },

                p { "swipe ", b { "up" }, " to ", b { "log in" } },
                br {}, br {},
                pre { class: "ascii-logo", "{ascii_logo}" },
                br {}, br {},
                p { "swipe ", b { "down" }, " to ", b { "create profile" } },

                div {
                    class : "home-direction-arrow",
                    "↓"
                }
            }
        }

        Register {swipe_state}
    }
}
