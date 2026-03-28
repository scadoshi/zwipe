use dioxus::prelude::*;
use crate::{Footer, Nav};

#[component]
pub fn Contribute() -> Element {
    rsx! {
        Nav {}
        div { class: "page",
            div { class: "coming-soon",
                h1 { "contribute" }
                h2 { "coming soon" }
                p {
                    "zwipe is a solo indie project. if you're enjoying it, support is
                    always appreciated — it goes directly toward server costs and
                    keeping development moving."
                }
                p {
                    "stripe donations will be available here shortly."
                }
            }
        }
        Footer {}
    }
}
