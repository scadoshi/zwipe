use crate::{DISCORD_URL, Footer, Nav, components::PageMeta};
use dioxus::prelude::*;

#[component]
pub fn Discord() -> Element {
    rsx! {
        PageMeta {
            title: "Discord",
            description: "Join the Zwipe Discord community. Connect with other users, report bugs, suggest enhancements, and follow development updates.",
            path: "/discord",
        }
        Nav {}
        div { class: "page content-enter",
            div { class: "section panel",
                h1 { "Join the Community" }
                p {
                    "Connect with other Zwipe users, report bugs, request help, "
                    "suggest enhancements, and follow development updates."
                }
                p {
                    a {
                        href: "{DISCORD_URL}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "store-btn",
                        "Join the Discord ↗"
                    }
                }
            }
        }
        Footer {}
    }
}
