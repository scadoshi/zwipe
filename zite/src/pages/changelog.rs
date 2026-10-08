use crate::{Footer, Nav, components::PageMeta};
use dioxus::prelude::*;
use zwipe_components::{Changelog as ChangelogContent, Panel};

#[component]
pub fn Changelog() -> Element {
    rsx! {
        PageMeta {
            title: "Changelog",
            description: "Every Zwipe release, newest first: what changed in each version of the app.",
            path: "/changelog",
        }
        Nav {}
        div { class: "page content-enter",
            div { class: "page-header",
                Panel { title: "Changelog", title_h1: true,
                    p { class: "tagline", "Every release, newest first." }
                }
            }
            ChangelogContent {}
        }
        Footer {}
    }
}
