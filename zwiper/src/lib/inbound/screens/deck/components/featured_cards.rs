//! Featured strip at the top of the deck screen: command zone + MVPs as card
//! images on one line, each labeled: the deck's identity at a glance.
//!
//! Mirrors the zite share page's featured row (`sd-featured`), sharing the
//! [`FlippableCardImage`] so DFC commanders can show both faces. Cards share
//! the row's width and shrink together instead of wrapping.

use std::time::Duration;

use dioxus::prelude::*;
use tokio::time;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::ImageSize;

use super::super::card::components::flippable_card_image::FlippableCardImage;

/// Seconds the row waits for the card images before it opens anyway.
const REVEAL_TIMEOUT_SECS: u64 = 4;

/// One labeled featured card: image (when available) above name + role chip.
/// Tapping it fires `on_tap` with the card, and the tap stops at the card so
/// the screen's tap-off doesn't see it. Nothing covers a featured card, so it
/// never floats: the first tap acts. `on_load` fires once the image has its
/// bytes, or at once when the card has none.
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

/// The featured row. While the deck loads it is a ghost at the row's height
/// when `ghost` says the deck likely features a card, else nothing; once
/// loaded, nothing when there is nothing to feature.
///
/// A row that opens on a page already loaded holds zero height until every
/// card's image has loaded, or 4 s pass, then opens and deals the cards in.
/// A row that follows its ghost is open from the start, its cards in place
/// over ghost image boxes that their art fills as it loads, so nothing under
/// it moves. Once open it stays open, so a card starred later joins without
/// a re-deal of the row.
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
    let mut images_loaded: Signal<usize> = use_signal(|| 0);
    let mut timed_out: Signal<bool> = use_signal(|| false);
    let mut opened: Signal<bool> = use_signal(|| false);
    // Whether the row first drew as its ghost, so it opens in place.
    let held = use_hook(|| !loaded && ghost);
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
    let class = match (held, opened()) {
        (true, _) => "deck-featured held",
        (false, true) => "deck-featured ready",
        (false, false) => "deck-featured",
    };
    rsx! {
        section { class: "{class}",
            div { class: "deck-featured-row scroll-fade-x", "data-settle-scope": "",
                for (card, role) in cards {
                    FeaturedCard {
                        key: "{card.scryfall_data.id}",
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

/// The row's ghost: one card's image box and its name and role chips, at the
/// row's open height.
#[component]
fn FeaturedGhost() -> Element {
    rsx! {
        section { class: "deck-featured held", "aria-hidden": "true",
            div { class: "deck-featured-row",
                div { class: "deck-featured-card",
                    div { class: "deck-featured-image" }
                    div { class: "deck-featured-chips",
                        span { class: "skeleton-bar skeleton-chip skeleton-chip-md" }
                        span { class: "skeleton-bar skeleton-chip skeleton-chip-sm" }
                    }
                }
            }
        }
    }
}
