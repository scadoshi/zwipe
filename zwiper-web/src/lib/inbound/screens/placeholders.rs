//! A panel naming each screen that is not built yet, so every route and
//! sidebar link works from the start. A wave replaces its screens here with
//! modules of their own.

use components::Panel;
use dioxus::prelude::*;
use uuid::Uuid;

use crate::inbound::shell::ScreenFrame;

/// A screen's title over a panel saying it is still to come, naming the
/// deck when the route carries one.
#[component]
fn Placeholder(title: String, #[props(default)] deck_id: Option<Uuid>) -> Element {
    rsx! {
        ScreenFrame { title: title.clone(),
            div { class: "web-placeholder content-enter",
                Panel { title,
                    p { "This screen is not built for the web yet." }
                    if let Some(deck_id) = deck_id {
                        p { class: "text-muted", "Deck {deck_id}" }
                    }
                }
            }
        }
    }
}

/// Home, not built yet.
#[component]
pub fn Home() -> Element {
    rsx! { Placeholder { title: "Home" } }
}

/// Privacy policy, not built yet.
#[component]
pub fn PrivacyPolicy() -> Element {
    rsx! { Placeholder { title: "Privacy policy" } }
}

/// Changelog, not built yet.
#[component]
pub fn Changelog() -> Element {
    rsx! { Placeholder { title: "Changelog" } }
}

/// Activity, not built yet.
#[component]
pub fn Activity() -> Element {
    rsx! { Placeholder { title: "Activity" } }
}

/// Decks, not built yet.
#[component]
pub fn DeckList() -> Element {
    rsx! { Placeholder { title: "Decks" } }
}

/// Commander maybeboard, not built yet.
#[component]
pub fn CommanderMaybeboard() -> Element {
    rsx! { Placeholder { title: "Commander maybeboard" } }
}

/// Playstyle, not built yet.
#[component]
pub fn Playstyle() -> Element {
    rsx! { Placeholder { title: "Playstyle" } }
}

/// Create deck, not built yet.
#[component]
pub fn CreateDeck() -> Element {
    rsx! { Placeholder { title: "Create deck" } }
}

/// Edit deck for one deck, not built yet.
#[component]
pub fn EditDeck(deck_id: Uuid) -> Element {
    rsx! { Placeholder { title: "Edit deck", deck_id } }
}

/// Deck for one deck, not built yet.
#[component]
pub fn ViewDeck(deck_id: Uuid) -> Element {
    rsx! { Placeholder { title: "Deck", deck_id } }
}

/// Import for one deck, not built yet.
#[component]
pub fn ImportDeck(deck_id: Uuid) -> Element {
    rsx! { Placeholder { title: "Import", deck_id } }
}

/// Export for one deck, not built yet.
#[component]
pub fn ExportDeck(deck_id: Uuid) -> Element {
    rsx! { Placeholder { title: "Export", deck_id } }
}

/// Add cards for one deck, not built yet.
#[component]
pub fn AddDeckCard(deck_id: Uuid) -> Element {
    rsx! { Placeholder { title: "Add cards", deck_id } }
}

/// Cards for one deck, not built yet.
#[component]
pub fn ViewDeckCard(deck_id: Uuid) -> Element {
    rsx! { Placeholder { title: "Cards", deck_id } }
}

/// Remove cards for one deck, not built yet.
#[component]
pub fn RemoveDeckCard(deck_id: Uuid) -> Element {
    rsx! { Placeholder { title: "Remove cards", deck_id } }
}

/// A path no route matches.
#[component]
pub fn NotFound(segments: Vec<String>) -> Element {
    let path = segments.join("/");
    rsx! {
        div { class: "web-public",
            div { class: "web-placeholder content-enter",
                Panel { title: "Not found",
                    p { "Nothing lives at /{path}." }
                    Link { to: "/", "Home" }
                }
            }
        }
    }
}
