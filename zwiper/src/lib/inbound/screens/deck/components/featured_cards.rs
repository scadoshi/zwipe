//! Featured strip at the top of the deck screen: command zone + MVPs as card
//! images on one line, each labeled: the deck's identity at a glance.
//!
//! Mirrors the zite share page's featured row (`sd-featured`), sharing the
//! [`FlippableCardImage`] so DFC commanders can show both faces. Cards share
//! the row's width and shrink together instead of wrapping.

use std::time::Duration;

use ::components::screen_return::ScreenReturns;
use dioxus::prelude::*;
use tokio::time;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::ImageSize;

use super::super::card::components::flippable_card_image::FlippableCardImage;

/// How long a deal takes, in milliseconds: the sixth card's start (0.12s plus five 0.12s steps)
/// and its 0.6s drop (`screens-b.css`, `.deck-featured.dealing`).
const DEAL_MS: u64 = 1_400;

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
/// fades in, so nothing under it moves. Each time it mounts with its cards (the
/// Cards layer opening), and each time its screen comes back from under another
/// ([`ScreenReturns`]), they deal into that held space, opacity and transform
/// only; a card starred after the deal just joins.
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
    let mut dealt = use_signal(|| false);
    // Each deal outdates the one before it, so a return mid-deal restarts it cleanly.
    let mut round = use_signal(|| 0u32);
    let dealing = loaded && !cards.is_empty();
    // The screen the row is on coming back from under another deals the row again.
    let returns = try_use_context::<ScreenReturns>().map_or(0, |ScreenReturns(r)| r());
    use_effect(use_reactive!(|dealing, returns| {
        let fresh = *round.peek() == 0 || returns > 0;
        if dealing && (fresh || !*dealt.peek()) {
            let this = round.peek().wrapping_add(1);
            round.set(this);
            dealt.set(false);
            spawn(async move {
                time::sleep(Duration::from_millis(DEAL_MS)).await;
                if *round.peek() == this {
                    dealt.set(true);
                }
            });
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
    rsx! {
        section { class: if dealt() { "deck-featured" } else { "deck-featured dealing" },
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
