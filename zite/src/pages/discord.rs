use crate::{DISCORD_URL, Footer, Nav, components::PageMeta};
use dioxus::prelude::*;
use zwipe_components::Panel;

#[component]
pub fn Discord() -> Element {
    rsx! {
        PageMeta {
            title: "Discord",
            description: "Join the Zwipe Discord: talk decks, report bugs, suggest features, and follow development.",
            path: "/discord",
        }
        Nav {}
        div { class: "page content-enter",
            div { class: "section",
                Panel { eyebrow: "Community", title: "Join the Community", title_h1: true,
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
        }
        Footer {}
    }
}
