//! Featured strip at the top of the deck screen: command zone + MVPs as card
//! images on one line, each labeled — the deck's identity at a glance.
//!
//! Mirrors the zite share page's featured row (`sd-featured`), sharing the
//! [`FlippableCardImage`] so DFC commanders can show both faces. Cards share
//! the row's width and shrink together instead of wrapping.

use super::super::card::components::flippable_card_image::FlippableCardImage;
use dioxus::prelude::*;
use zwipe_core::domain::card::{Card, scryfall_data::ImageSize};

/// One labeled featured card: image (when available) above name + role chip.
/// Tapping it fires `on_tap` with the card so the host can open its image
/// overlay.
#[component]
fn FeaturedCard(card: Card, role: String, on_tap: EventHandler<Card>) -> Element {
    let name = card.scryfall_data.name.clone();
    let sd = card.scryfall_data.clone();
    let has_image = sd.primary_image_url(ImageSize::Normal).is_some();
    rsx! {
        div {
            class: "deck-featured-card",
            onclick: move |_| on_tap.call(card.clone()),
            if has_image {
                FlippableCardImage {
                    sd,
                    size: ImageSize::Normal,
                    class: "deck-featured-image".to_string(),
                    draggable: false,
                }
            }
            div { class: "deck-featured-name", "{name}" }
            if !role.is_empty() {
                div {
                    class: if role == "MVP" { "deck-featured-role deck-featured-role-mvp" } else { "deck-featured-role deck-featured-role-zone" },
                    "{role}"
                }
            }
        }
    }
}

/// The featured row. Renders nothing when there is nothing to feature.
#[component]
pub fn FeaturedCards(
    /// `(card, role label)` pairs, command zone first then MVPs.
    cards: Vec<(Card, String)>,
    /// Fired with the tapped card — the host opens its image overlay.
    on_tap: EventHandler<Card>,
) -> Element {
    if cards.is_empty() {
        return rsx! {};
    }
    rsx! {
        section { class: "deck-featured",
            for (card, role) in cards {
                FeaturedCard { key: "{card.scryfall_data.id}", card, role, on_tap }
            }
        }
    }
}
