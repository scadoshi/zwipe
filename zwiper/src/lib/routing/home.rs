use dioxus::prelude::*;

#[component]
pub fn Home() -> Element {
    rsx! {
        div { class : "home-directions",
            div { class : "home-direction-arrow",
            "🡅"
            }
            p {
                "swipe ", b { "up" }, " to ", b { "log in" }
            },
            br {}, br {}, br {}
            p {
                "swipe ", b { "down" }, " to ", b { "create profile" }
            },
            div { class : "home-direction-arrow",
            "🡇"
            }
        }
    }
}
