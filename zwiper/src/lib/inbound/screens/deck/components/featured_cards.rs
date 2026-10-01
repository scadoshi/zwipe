//! Featured strip at the top of the deck screen: command zone + MVPs as card
//! images on one line, each labeled: the deck's identity at a glance.
//!
//! Mirrors the zite share page's featured row (`sd-featured`), sharing the
//! [`FlippableCardImage`] so DFC commanders can show both faces. Cards share
//! the row's width and shrink together instead of wrapping.

use super::super::card::components::flippable_card_image::FlippableCardImage;
use dioxus::prelude::*;
use std::time::Duration;
use tokio::time::sleep;
use zwipe_core::domain::card::{Card, scryfall_data::ImageSize};

/// One labeled featured card: image (when available) above name + role chip.
/// Tapping it fires `on_tap` with the card so the host can open its image
/// overlay. `on_load` fires once the image has its bytes, or at once when the
/// card has none.
#[component]
fn FeaturedCard(
    card: Card,
    role: String,
    on_tap: EventHandler<Card>,
    on_load: EventHandler<()>,
) -> Element {
    let name = card.scryfall_data.name.clone();
    let sd = card.scryfall_data.clone();
    let has_image = sd.primary_image_url(ImageSize::Normal).is_some();
    use_effect(use_reactive!(|has_image| {
        if !has_image {
            on_load.call(());
        }
    }));
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
                    on_load: move |_| on_load.call(()),
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

/// The featured row. Renders nothing until the deck has loaded or when there
/// is nothing to feature. The row holds zero height until every card's image
/// has loaded, or 4 s pass, then opens and deals the cards in. Once open it
/// stays open, so a card starred later joins without a re-deal of the row.
#[component]
pub fn FeaturedCards(
    /// `(card, role label)` pairs, command zone first then MVPs.
    cards: Vec<(Card, String)>,
    /// Whether the deck's cards and command zone have all arrived.
    loaded: bool,
    /// Fired with the tapped card: the host opens its image overlay.
    on_tap: EventHandler<Card>,
) -> Element {
    let mut images_loaded: Signal<usize> = use_signal(|| 0);
    let mut timed_out: Signal<bool> = use_signal(|| false);
    let mut opened: Signal<bool> = use_signal(|| false);
    use_future(move || async move {
        sleep(Duration::from_secs(4)).await;
        timed_out.set(true);
    });
    let total = cards.len();
    use_effect(use_reactive!(|total| {
        if !opened() && (timed_out() || images_loaded() >= total) {
            opened.set(true);
        }
    }));
    if !loaded || cards.is_empty() {
        return rsx! {};
    }
    let class = if opened() {
        "deck-featured ready"
    } else {
        "deck-featured"
    };
    rsx! {
        section { class: "{class}",
            div { class: "deck-featured-row",
                for (card, role) in cards {
                    FeaturedCard {
                        key: "{card.scryfall_data.id}",
                        card,
                        role,
                        on_tap,
                        on_load: move |_| images_loaded += 1,
                    }
                }
            }
        }
    }
}
