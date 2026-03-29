use dioxus::prelude::*;
use crate::{Footer, Nav};

#[component]
pub fn Download() -> Element {
    rsx! {
        Nav {}
        div { class: "page content-enter",
            div { class: "section",
                h1 { "application pending" }
                p {
                    "zwipe has been submitted to the app store and is currently under review. "
                    "once approved, this page will redirect to the official download link."
                }
                p { "check back soon." }
            }
        }
        Footer {}
    }
}
