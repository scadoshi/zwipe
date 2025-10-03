use dioxus::prelude::*;
use zwipe::domain::ascii_logo;

use crate::{
    routing::Route,
    swipe::{
        self, handle_onmousedown, handle_onmousemove, handle_onmouseup, handle_ontouchend,
        handle_ontouchmove, handle_ontouchstart,
    },
};

#[component]
pub fn Home() -> Element {
    let navigator = use_navigator();
    let ascii_logo = ascii_logo::logo();
    let mut swipe_state = use_signal(|| swipe::State::new());

    rsx! {
        div { class : "swipe-able",

            style : format!("transform: translateY({}px);", -swipe_state.read().dy),

            ontouchstart : move |e: Event<TouchData>| handle_ontouchstart(e, &mut swipe_state),
            ontouchmove : move |e: Event<TouchData>| handle_ontouchmove(e, &mut swipe_state),
            ontouchend : move |e: Event<TouchData>| handle_ontouchend(e, &mut swipe_state),

            onmousedown : move |e: Event<MouseData>| handle_onmousedown(e, &mut swipe_state),
            onmousemove : move |e: Event<MouseData>| handle_onmousemove(e, &mut swipe_state),
            onmouseup : move |e: Event<MouseData>| handle_onmouseup(e, &mut swipe_state),

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
