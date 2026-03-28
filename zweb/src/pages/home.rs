use dioxus::prelude::*;
use crate::{APP_STORE_URL, Footer, Nav};

const LOGO_ASCII: &str = include_str!("../../assets/logo.txt");

#[component]
pub fn Home() -> Element {
    rsx! {
        Nav {}
        div { class: "hero",
            div { class: "logo", "{LOGO_ASCII}" }
            p { class: "tagline",
                "the mtg deck builder built for mobile. swipe right to add, left to skip."
            }
            a { href: APP_STORE_URL, class: "appstore-btn", "download on the app store" }
        }
        div { class: "page",
            div { class: "features-grid",
                div { class: "feature-card",
                    h3 { "swipe to build" }
                    p { "browse cards one at a time. right to add, left to skip. no clutter." }
                }
                div { class: "feature-card",
                    h3 { "deep filters" }
                    p { "filter by color, type, mana cost, oracle text, artist, set, and more." }
                }
                div { class: "feature-card",
                    h3 { "35k+ cards" }
                    p { "full scryfall card database, synced nightly. every card, every set." }
                }
                div { class: "feature-card",
                    h3 { "commander ready" }
                    p { "singleton mode, commander assignment, mana curve and color identity." }
                }
                div { class: "feature-card",
                    h3 { "import / export" }
                    p { "paste any decklist from moxfield or archidekt and it just works." }
                }
                div { class: "feature-card",
                    h3 { "your decks, synced" }
                    p { "account-based. your decks are always there across sessions." }
                }
            }
        }
        Footer {}
    }
}
