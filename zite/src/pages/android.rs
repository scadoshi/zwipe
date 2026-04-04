use dioxus::prelude::*;
use crate::{Footer, Nav};

#[component]
pub fn Android() -> Element {
    rsx! {
        Nav {}
        div { class: "page content-enter",
            div { class: "section",
                h1 { "google play — pending" }
                p {
                    "zwipe has been submitted to google play and is currently under review. "
                    "once approved, this page will redirect to the official download link."
                }
                p { "check back soon." }
            }
        }
        Footer {}
    }
}
