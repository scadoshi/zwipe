use dioxus::prelude::*;
use zwipe::domain::ascii_logo;

use crate::routing::Route;

#[component]
pub fn Home() -> Element {
    let navigator = use_navigator();
    let ascii_logo = ascii_logo::logo();

    rsx! {

        div { class : "home",
            div {
                onclick : move |_| {
                    navigator.push(Route::Login {});
                },
                class : "home-direction-arrow",
                "↑"
            }
            p {
                "swipe ", b { "up" }, " to ", b { "log in" }
            },
            br {}, br {}, pre { class: "ascii-logo", "{ascii_logo}" }, br {}, br {}
            p {
                "swipe ", b { "down" }, " to ", b { "create profile" }
            },
            div {
                onclick : move |_| {
                    navigator.push(Route::Register {});
                },
                class : "home-direction-arrow",
                "↓"
            }
        }
    }
}
