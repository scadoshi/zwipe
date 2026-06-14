use dioxus::prelude::*;
use crate::components::PageMeta;
use crate::{Footer, Nav};

#[component]
pub fn Android() -> Element {
    rsx! {
        PageMeta {
            title: "Download for Android",
            description: "Zwipe for Android: swipe-based Magic: The Gathering deck builder. Currently in Google Play review. Commander support, 110k+ cards, account-synced decks.",
            path: "/download/android",
        }
        Nav {}
        div { class: "page content-enter",
            div { class: "section panel",
                h1 { "Google Play: Pending" }
                p {
                    "Zwipe has been submitted to Google Play. While it's under review, here's
                    what's waiting for you: swipe-based deck building, Commander support with
                    partners and backgrounds, sideboard, maybeboard, 14 themes with dark/light modes,
                    multiple printings per card, and 110k+ cards synced nightly."
                }
                p { "Once approved, this page will redirect to the official download link." }
            }
        }
        Footer {}
    }
}
