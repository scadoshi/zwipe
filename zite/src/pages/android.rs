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
                    "zwipe has been submitted to google play. while it's under review, here's
                    what's waiting for you: swipe-based deck building, commander support with
                    partners and backgrounds, maybeboard, 9 themes, and 110k+ cards synced nightly."
                }
                p { "once approved, this page will redirect to the official download link." }
            }
        }
        Footer {}
    }
}
