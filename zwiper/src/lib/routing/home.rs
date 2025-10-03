use dioxus::prelude::*;
use zwipe::domain::ascii_logo;

use crate::{
    routing::Route,
    swipe::{self, OnMouse, OnTouch},
};

#[component]
pub fn Home() -> Element {
    let navigator = use_navigator();
    let ascii_logo = ascii_logo::logo();
    let mut swipe_state = use_signal(|| swipe::State::new());

    rsx! {
        div { class : "swipe-able",

            style : format!(
                "transform: translateY({}px);
                transition: transform {}s;",
                -swipe_state.read().dy(),
                swipe_state.read().transition_seconds
            ),

            ontouchstart : move |e: Event<TouchData>| swipe_state.handle_ontouchstart(e),
            ontouchmove : move |e: Event<TouchData>| swipe_state.handle_ontouchmove(e),
            ontouchend : move |e: Event<TouchData>| swipe_state.handle_ontouchend(e),

            onmousedown : move |e: Event<MouseData>| swipe_state.handle_onmousedown(e),
            onmousemove : move |e: Event<MouseData>| swipe_state.handle_onmousemove(e),
            onmouseup : move |e: Event<MouseData>| swipe_state.handle_onmouseup(e),

            div { class : "home-screen",
                div {
                    // onclick : move |_| {
                    //     navigator.push(Route::Login {});
                    // },
                    class : "home-direction-arrow",
                    "↑"
                },

                p { "swipe ", b { "up" }, " to ", b { "log in" } },
                br {}, br {},
                pre { class: "ascii-logo", "{ascii_logo}" },
                br {}, br {},
                p { "swipe ", b { "down" }, " to ", b { "create profile" } },

                div {
                    // onclick : move |_| {
                    //     navigator.push(Route::Register {});
                    // },
                    class : "home-direction-arrow",
                    "↓"
                }
            }
        }
    }
}
