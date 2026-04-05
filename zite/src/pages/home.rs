use dioxus::prelude::*;
use crate::{Footer, Nav, Route};

const LOGO_ASCII: &str = zwipe_core::domain::logo::ZWIPE;

#[component]
pub fn Home() -> Element {
    rsx! {
        Nav {}
        div { class: "hero",
            div { class: "logo", "{LOGO_ASCII}" }
            p { class: "tagline",
                "the mtg deck builder built for mobile. swipe right to add, left to skip, up to maybe."
            }
            div { class: "store-buttons",
                Link { to: Route::Ios {}, class: "store-btn", "download on the app store" }
                Link { to: Route::Android {}, class: "store-btn", "download on google play" }
            }
        }
        div { class: "page content-enter",
            div { class: "features-grid",
                div { class: "feature-card",
                    h3 { "swipe to build" }
                    p { "browse cards one at a time. right to add, left to skip, up for maybeboard." }
                }
                div { class: "feature-card",
                    h3 { "deep filters" }
                    p { "filter by color, type, mana cost, oracle text, keywords, artist, set, rarity, commander eligibility, and more. per-section clear for fine-tuned control." }
                }
                div { class: "feature-card",
                    h3 { "110k+ cards" }
                    p { "every english printing from scryfall, synced nightly. multiple printings per card — pick your favorite art." }
                }
                div { class: "feature-card",
                    h3 { "commander ready" }
                    p { "full commander support — partners, backgrounds, oathbreaker with signature spell. eligibility filtering per format. color identity validation across the command zone." }
                }
                div { class: "feature-card",
                    h3 { "import / export" }
                    p { "paste any decklist from moxfield or archidekt. maybeboard sections import and export automatically." }
                }
                div { class: "feature-card",
                    h3 { "your decks, synced" }
                    p { "account-based. your decks are always there across sessions." }
                }
                div { class: "feature-card",
                    h3 { "maybeboard" }
                    p { "stage cards you're considering without committing them to the deck. swipe up to maybe, move to deck when ready." }
                }
                div { class: "feature-card",
                    h3 { "9 themes" }
                    p { "dark mode by default. choose from 9 color themes to match your style." }
                }
            }
        }
        Footer {}
    }
}
