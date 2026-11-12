//! Featured strip at the top of the deck screen: command zone + MVPs as card
//! images on one line, each labeled: the deck's identity at a glance.
//!
//! Mirrors the zite share page's featured row (`sd-featured`), sharing the
//! [`FlippableCardImage`] so DFC commanders can show both faces. Cards share
//! the row's width and shrink together instead of wrapping.

use std::time::Duration;

use dioxus::prelude::*;
use tokio::time;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::ImageSize;

use super::super::card::components::flippable_card_image::FlippableCardImage;

/// Seconds the row waits for the card images before it opens anyway.
const REVEAL_TIMEOUT_SECS: u64 = 4;

/// One labeled featured card: image (when available) above name + role chip.
/// Tapping it fires `on_tap` with the card, and the tap stops at the card so
/// the screen's tap-off doesn't see it. While `floating` (revealed by a first
/// tap) the image floats as a revealed stack card does. `on_load` fires once
/// the image has its bytes, or at once when the card has none.
#[component]
fn FeaturedCard(
    card: Card,
    role: String,
    floating: bool,
    on_tap: EventHandler<Card>,
    on_load: EventHandler<()>,
) -> Element {
    let name = card.scryfall_data.name.clone();
    let sd = card.scryfall_data.clone();
    let has_image = sd.primary_image_url(ImageSize::Normal).is_some();
    let image_class = if floating {
        "deck-featured-image floatable floating"
    } else {
        "deck-featured-image floatable"
    };
    use_effect(use_reactive!(|has_image| {
        if !has_image {
            on_load.call(());
        }
    }));
    rsx! {
        div {
            class: "deck-featured-card",
            onclick: move |event: MouseEvent| {
                event.stop_propagation();
                on_tap.call(card.clone());
            },
            if has_image {
                FlippableCardImage {
                    sd,
                    size: ImageSize::Normal,
                    class: image_class.to_string(),
                    draggable: false,
                    on_load: move |()| on_load.call(()),
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
    /// Fired with the tapped card: the host reveals it, opens its details or
    /// opens its image.
    on_tap: EventHandler<Card>,
    /// The card a first tap revealed, which floats until a tap elsewhere.
    #[props(default)]
    revealed: Option<Uuid>,
) -> Element {
    let mut images_loaded: Signal<usize> = use_signal(|| 0);
    let mut timed_out: Signal<bool> = use_signal(|| false);
    let mut opened: Signal<bool> = use_signal(|| false);
    use_future(move || async move {
        time::sleep(Duration::from_secs(REVEAL_TIMEOUT_SECS)).await;
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
            div { class: "deck-featured-row scroll-fade-x",
                for (card, role) in cards {
                    FeaturedCard {
                        key: "{card.scryfall_data.id}",
                        floating: revealed == Some(card.scryfall_data.id),
                        card,
                        role,
                        on_tap,
                        on_load: move |()| images_loaded += 1,
                    }
                }
            }
        }
    }
}
