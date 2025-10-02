use dioxus::prelude::*;

use crate::routing::Route;

#[component]
pub fn Home() -> Element {
    let navigator = use_navigator();
    rsx! {

        div { class : "home-directions",
            div {
                onclick : move |_| {
                    navigator.push(Route::Login {});
                },
                class : "home-direction-arrow",
                "🡅"
            }
            p {
                "swipe ", b { "up" }, " to ", b { "log in" }
            },
            br {}, br {}, br {}, br {}
            p {
                "swipe ", b { "down" }, " to ", b { "create profile" }
            },
            div {
                onclick : move |_| {
                    navigator.push(Route::Register {});
                },
                class : "home-direction-arrow",
                "🡇"
            }
        }
    }
}
