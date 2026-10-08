use crate::{DISCORD_URL, Footer, Nav, SUPPORT_EMAIL, components::PageMeta};
use dioxus::prelude::*;
use zwipe_components::Panel;

#[component]
pub fn Community() -> Element {
    rsx! {
        PageMeta {
            title: "Community",
            description: "Get help with Zwipe, report bugs, suggest features, and follow updates on Discord or by email.",
            path: "/community",
        }
        Nav {}
        div { class: "page content-enter",
            div { class: "section",
                Panel { title: "Community", title_h1: true,
                    p { "Get help, report bugs, suggest features, and follow updates." }
                }
            }

            div { class: "section",
                div { class: "card-grid",
                    Panel {
                        eyebrow: "Chat",
                        title: "Discord",
                        actions: rsx! {
                            a {
                                class: "panel-action",
                                href: "{DISCORD_URL}",
                                "Join the Discord" span { class: "ext", "\u{2197}" }
                            }
                        },
                        p { class: "card-summary", "Talk decks with other players and follow what's shipping." }
                    }
                    Panel {
                        eyebrow: "Support",
                        title: "Email",
                        actions: rsx! {
                            a {
                                class: "panel-action",
                                href: "mailto:{SUPPORT_EMAIL}",
                                "Email support" span { class: "ext", "\u{2197}" }
                            }
                        },
                        p { class: "card-summary",
                            "For account problems or anything private, email "
                            a { href: "mailto:{SUPPORT_EMAIL}", "{SUPPORT_EMAIL}" }
                            "."
                        }
                    }
                }
            }
        }
        Footer {}
    }
}
