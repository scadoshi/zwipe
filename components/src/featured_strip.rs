//! The featured strip at the top of a deck's Cards on a phone: the command zone and the main
//! board's MVPs as card images on one line, each labeled, the deck's identity at a glance. The
//! app draws it, and the web does at phone width, so both are the same strip.
//!
//! Every card takes the same width, two to the strip's width up to 11rem, whatever the count, so
//! the strip's height never changes when a card joins or leaves; a third and later card scrolls
//! in from the faded edge. The strip holds that height from its first frame, each image box
//! ghosted until its art fades in, so nothing under it moves while the deck and its images land.

use dioxus::prelude::*;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::ImageSize;

use crate::flippable_card_image::FlippableCardImage;
use crate::screen_return::ScreenReturns;

/// The deal's keyframes, as the stylesheet names them (`.deck-featured.dealing`).
const DEAL_ANIMATION: &str = "deck-featured-deal";

/// One labeled featured card: its image (when it has one) over its name and role chips.
/// Tapping it fires `on_tap` with the card, and the tap stops at the card so the screen's tap-off
/// doesn't see it. Nothing covers a featured card, so it never floats: the first tap acts.
#[component]
fn StripCard(card: Card, role: String, on_tap: EventHandler<Card>) -> Element {
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

/// The strip. While the deck loads it is a ghost at the strip's height when `ghost` says the deck
/// likely features a card, else nothing; once loaded, nothing when there is nothing to feature.
///
/// Its cards deal into the held space (opacity and transform only) when it mounts with them, each
/// time `deal_key` changes, and each time its screen comes back from under another
/// ([`ScreenReturns`]); the deal ends on its last card's `animationend`, after which a card
/// starred later just joins.
#[component]
pub fn FeaturedStrip(
    /// `(card, role label)` pairs, command zone first then MVPs.
    cards: Vec<(Card, String)>,
    /// Whether the deck's cards and command zone have all arrived.
    loaded: bool,
    /// Whether to hold the strip's place with a ghost while the deck loads.
    #[props(default)]
    ghost: bool,
    /// A host's count of entries: each change deals the cards again.
    #[props(default)]
    deal_key: u32,
    /// Fired with the tapped card: the host opens its details or its image.
    on_tap: EventHandler<Card>,
) -> Element {
    let mut dealing = use_signal(|| true);
    let mut landed = use_signal(|| 0usize);
    let returns = try_use_context::<ScreenReturns>().map_or(0, |ScreenReturns(r)| r());
    use_effect(use_reactive!(|deal_key, returns| {
        let _ = (deal_key, returns);
        landed.set(0);
        dealing.set(true);
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
    let count = cards.len();
    rsx! {
        section {
            class: if dealing() { "deck-featured dealing" } else { "deck-featured" },
            onanimationend: move |event: AnimationEvent| {
                if event.animation_name() == DEAL_ANIMATION {
                    let now = landed.peek().saturating_add(1);
                    landed.set(now);
                    if now >= count {
                        dealing.set(false);
                    }
                }
            },
            div { class: "deck-featured-row scroll-fade-x", "data-settle-scope": "",
                for (card, role) in cards {
                    StripCard { key: "{card.scryfall_data.id}", card, role, on_tap }
                }
            }
        }
    }
}

/// The strip's ghost: one card's image box and its name and role chips, at the strip's height.
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
