use dioxus::prelude::*;
use crate::{Footer, Nav};

#[component]
pub fn Discord() -> Element {
    rsx! {
        Nav {}
        div { class: "page content-enter",
            div { class: "section",
                h1 { "join the community" }
                p {
                    "connect with other zwipe users, report bugs, request help, "
                    "suggest enhancements, and follow development updates."
                }
                p {
                    a {
                        href: "https://discord.gg/s2UReqUUeg",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "store-btn",
                        "join the discord ↗"
                    }
                }
            }
        }
        Footer {}
    }
}
