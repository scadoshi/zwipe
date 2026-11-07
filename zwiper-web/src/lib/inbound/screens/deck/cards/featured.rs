//! The deck cards view's featured strip: the command zone with each card's
//! role, then the main board's MVPs, oldest star first. The row stays shut
//! until every image has loaded or a few seconds pass, then deals the cards
//! in; a click opens the card's image.

use std::time::Duration;

use dioxus::prelude::*;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::ImageSize;

use crate::outbound::timer;

/// How long the strip waits on its images before dealing the cards in.
const REVEAL_TIMEOUT: Duration = Duration::from_secs(4);

/// One featured card: its image, name and role chip.
#[component]
fn FeaturedCard(
    card: Card,
    role: &'static str,
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
    let role_class = if role == "MVP" {
        "stat-chip deck-featured-role-mvp"
    } else {
        "stat-chip deck-featured-role-zone"
    };
    rsx! {
        div { class: "deck-featured-card", onclick: move |_| on_tap.call(card.clone()),
            if has_image {
                components::FlippableCardImage {
                    sd,
                    size: ImageSize::Normal,
                    class: "deck-featured-image".to_string(),
                    draggable: false,
                    on_load: move |()| on_load.call(()),
                }
            }
            div { class: "deck-featured-chips",
                span { class: "stat-chip deck-featured-name", "{name}" }
                span { class: role_class, "{role}" }
            }
        }
    }
}

/// The strip, from zwipe-ui's `featured` order. Nothing shows until the deck
/// has loaded, or when there is nothing to feature.
#[component]
pub(super) fn FeaturedCards(
    cards: Vec<(Card, &'static str)>,
    loaded: bool,
    on_tap: EventHandler<Card>,
) -> Element {
    let mut images_loaded = use_signal(|| 0usize);
    let mut timed_out = use_signal(|| false);
    let mut opened = use_signal(|| false);
    use_future(move || async move {
        timer::sleep(REVEAL_TIMEOUT).await;
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
    rsx! {
        section { class: if opened() { "deck-featured ready" } else { "deck-featured" },
            div { class: "deck-featured-row scroll-fade-x",
                for (card, role) in cards {
                    FeaturedCard {
                        key: "{card.scryfall_data.id}",
                        card,
                        role,
                        on_tap,
                        on_load: move |()| {
                            let next = images_loaded.peek().saturating_add(1);
                            images_loaded.set(next);
                        },
                    }
                }
            }
        }
    }
}
