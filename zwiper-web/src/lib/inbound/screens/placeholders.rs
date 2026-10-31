//! A panel naming each screen that is not built yet, so every route and
//! sidebar link works from the start. A wave replaces its screens here with
//! modules of their own.

use components::{ActionBar, AlertDialog, Button, ButtonVariant, DialogAction, Panel};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;

use crate::inbound::session::logout;
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

/// Config, not built yet.
#[component]
pub fn Config() -> Element {
    rsx! { Placeholder { title: "Config" } }
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

/// Account, not built yet, with the app's `Log out` and its confirm dialog
/// so a session can end before Wave A lands.
#[component]
pub fn Account() -> Element {
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let mut confirm = use_signal(|| false);
    let log_out = use_callback(move |()| logout::logout(session, client));

    rsx! {
        ScreenFrame { title: "Account",
            div { class: "web-placeholder content-enter",
                Panel { title: "Account",
                    p { "This screen is not built for the web yet." }
                }
            }
            ActionBar {
                Button {
                    variant: ButtonVariant::Action,
                    danger: true,
                    onclick: move |_| confirm.set(true),
                    "Log out"
                }
            }
        }
        AlertDialog {
            open: confirm,
            title: "Log out",
            actions: vec![
                DialogAction::Dismiss("Cancel".into()),
                DialogAction::Confirm {
                    label: "Log out".into(),
                    danger: true,
                    on_confirm: log_out,
                },
            ],
            "Are you sure you want to log out?"
        }
    }
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
