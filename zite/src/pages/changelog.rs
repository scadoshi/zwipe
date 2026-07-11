use crate::{Footer, Nav, components::PageMeta};
use dioxus::prelude::*;
use zwipe_components::Changelog as ChangelogContent;

#[component]
pub fn Changelog() -> Element {
    rsx! {
        PageMeta {
            title: "Changelog",
            description: "Every Zwipe release, newest first. See what's new across versions of the mobile Magic: The Gathering deck builder.",
            path: "/changelog",
        }
        Nav {}
        div { class: "page content-enter",
            div { class: "page-header section panel",
                h1 { "Changelog" }
                p { class: "tagline", "Every release, newest first." }
            }
            ChangelogContent {}
        }
        Footer {}
    }
}
