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
