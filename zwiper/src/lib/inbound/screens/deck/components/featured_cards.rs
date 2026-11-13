//! Featured strip at the top of the deck screen: command zone + MVPs as card
//! images on one line, each labeled: the deck's identity at a glance.
//!
//! Mirrors the zite share page's featured row (`sd-featured`), sharing the
//! [`FlippableCardImage`] so DFC commanders can show both faces. Cards share
//! the row's width and shrink together instead of wrapping.

use dioxus::prelude::*;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::ImageSize;

use super::super::card::components::flippable_card_image::FlippableCardImage;

/// One labeled featured card: image (when available) above name + role chip.
/// Tapping it fires `on_tap` with the card, and the tap stops at the card so
/// the screen's tap-off doesn't see it. Nothing covers a featured card, so it
/// never floats: the first tap acts.
#[component]
fn FeaturedCard(card: Card, role: String, on_tap: EventHandler<Card>) -> Element {
    let name = card.scryfall_data.name.clone();
    let sd = card.scryfall_data.clone();
    let has_image = sd.primary_image_url(ImageSize::Normal).is_some();
    rsx! {
        div {
            class: "deck-featured-card",
            // Named for the settle, so the strip's cards slide when a star adds or takes one.
            "data-settle-key": "featured-{card.scryfall_data.id}",
            onclick: move |event: MouseEvent| {
                event.stop_propagation();
                on_tap.call(card.clone());
            },
            if has_image {
                FlippableCardImage {
                    sd,
                    size: ImageSize::Normal,
                    class: "deck-featured-image".to_string(),
                    draggable: false,
                }
            }
            div { class: "deck-featured-chips",
                span { class: "stat-chip deck-featured-name", "{name}" }
                if !role.is_empty() {
                    span {
                        class: if role == "MVP" { "stat-chip deck-featured-role-mvp" } else { "stat-chip deck-featured-role-zone" },
                        "{role}"
                    }
                }
            }
        }
    }
}

/// The featured row. While the deck loads it is a ghost at the row's height
/// when `ghost` says the deck likely features a card, else nothing; once
/// loaded, nothing when there is nothing to feature. The row holds its
/// height from its first frame, each card's image box ghosted until its art
/// fades in, so nothing under it moves.
#[component]
pub fn FeaturedCards(
    /// `(card, role label)` pairs, command zone first then MVPs.
    cards: Vec<(Card, String)>,
    /// Whether the deck's cards and command zone have all arrived.
    loaded: bool,
    /// Whether to hold the row's place with a ghost while the deck loads.
    #[props(default)]
    ghost: bool,
    /// Fired with the tapped card: the host opens its details or its image.
    on_tap: EventHandler<Card>,
) -> Element {
    if !loaded {
        return if ghost {
            rsx! { FeaturedGhost {} }
        } else {
            rsx! {}
        };
    }
    if cards.is_empty() {
        return rsx! {};
    }
    rsx! {
        section { class: "deck-featured",
            div { class: "deck-featured-row scroll-fade-x", "data-settle-scope": "",
                for (card, role) in cards {
                    FeaturedCard { key: "{card.scryfall_data.id}", card, role, on_tap }
                }
            }
        }
    }
}

/// The row's ghost: one card's image box and its name and role chips, at the
/// row's open height.
#[component]
fn FeaturedGhost() -> Element {
    rsx! {
        section { class: "deck-featured", "aria-hidden": "true",
            div { class: "deck-featured-row",
                div { class: "deck-featured-card",
                    div { class: "deck-featured-image" }
                    // Chip boxes at a loaded card's chips' height, filled as ghosts.
                    div { class: "deck-featured-chips",
                        span { class: "stat-chip deck-featured-ghost-chip", style: "width: 7rem;", "\u{a0}" }
                        span { class: "stat-chip deck-featured-ghost-chip", style: "width: 4.5rem;", "\u{a0}" }
                    }
                }
            }
        }
    }
}
