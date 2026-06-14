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
                h1 {
                    "Google Play: "
                    span { class: "accent-tertiary", "Pending" }
                }
                p {
                    "Zwipe is submitted and under review. Once approved, this page will
                    redirect to the official download link."
                }
            }
        }
        Footer {}
    }
}
