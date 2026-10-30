//! Featured flavor: the hour's shared flavor card, mirrored from the app.
//!
//! Same unauthed endpoint the app home screen reads (`FEATURED_FLAVOR_ROUTE`);
//! the server flips the pick at the top of every UTC hour, so the site shows
//! a living element that matches what app users see at the same moment.
//! Fetch pattern mirrors `StatsStrip`: hide until live data lands. Debug
//! builds additionally fall back to a frozen real response (below) so the
//! element always renders under `dx serve` with no backend.
//!
//! The card's name and price sit as tags in the panel's action row, beside
//! Image, TCGplayer and Card Kingdom buttons. Image requests the full-art
//! overlay via the `overlay` signal; the HOME PAGE renders the overlay itself
//! at its top level, outside the `content-enter` content tree, mirroring the
//! shared-deck page. Keeping the overlay nested here was the
//! trapped-in-the-page-column bug: an animated ancestor's transform makes it
//! the containing block for `position: fixed`.

use components::Panel;
use dioxus::prelude::*;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::{ImageSize, ScryfallData};
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::deck_metrics::card_price;

use crate::components::sleep_ms;

/// A real card response (Research Assistant, M15) frozen 2026-08-07, decoded as
/// the DEBUG-ONLY fallback while the live fetch is pending or failing, so
/// `dx serve` with no backend still renders the element. Release builds hide
/// the section until live data lands (StatsStrip parity): a stale-but-real
/// card is indistinguishable from a live one, which is exactly how the
/// CF-cached copy went unnoticed for a day (owner call 2026-08-06).
#[cfg(debug_assertions)]
const SPOOF_CARD_JSON: &str = include_str!("featured_flavor_spoof.json");

fn spoof_card() -> Option<Card> {
    #[cfg(debug_assertions)]
    return serde_json::from_str(SPOOF_CARD_JSON).ok();
    #[cfg(not(debug_assertions))]
    None
}

/// Dismisses the flavor overlay with the shared 200ms exit animation.
/// Split out so the home page's backdrop click handler and any future
/// dismiss affordance run the identical sequence.
pub(crate) fn dismiss_flavor_overlay(
    mut overlay: Signal<Option<ScryfallData>>,
    mut dismissing: Signal<bool>,
) {
    dismissing.set(true);
    spawn(async move {
        sleep_ms(200).await;
        overlay.set(None);
        dismissing.set(false);
    });
}

/// URL-encodes a card's front face name for a retailer link, as zwiper's
/// `buy_links` does.
fn encode_name(name: &str) -> String {
    name.split(" // ")
        .next()
        .unwrap_or(name)
        .replace(' ', "+")
        .replace(',', "%2C")
        .replace('\'', "%27")
}

/// The card's TCGplayer product page when its id is known, else a one-copy
/// mass entry by name.
fn tcgplayer_url(name: &str, tcgplayer_id: Option<i32>) -> String {
    match tcgplayer_id {
        Some(id) => format!("https://www.tcgplayer.com/product/{id}"),
        None => format!(
            "https://www.tcgplayer.com/massentry?c=1+{}",
            encode_name(name)
        ),
    }
}

/// A one-copy Card Kingdom builder link.
fn cardkingdom_url(name: &str) -> String {
    format!(
        "https://www.cardkingdom.com/builder?c=1+{}",
        encode_name(name)
    )
}

#[component]
pub fn FeaturedFlavor(overlay: Signal<Option<ScryfallData>>) -> Element {
    let fetched: Resource<Option<Card>> =
        use_resource(|| async { crate::api::client().featured_flavor().await.ok() });

    let value = fetched.read();
    let live = match &*value {
        Some(Some(card)) => Some(card.clone()),
        _ => None,
    };
    let Some(card) = live.or_else(spoof_card) else {
        return rsx! {};
    };
    let Some(flavor_text) = card.scryfall_data.flavor_text.clone() else {
        return rsx! {};
    };
    let sd = card.scryfall_data;
    let name = sd.name.clone();
    let has_image = sd.primary_image_url(ImageSize::Normal).is_some();
    let usd_price =
        card_price(&sd, PriceCurrency::Usd).map(|p| PriceCurrency::Usd.format_amount(p));
    let tcgplayer = tcgplayer_url(&name, sd.tcgplayer_id);
    let cardkingdom = cardkingdom_url(&name);

    rsx! {
        section { class: "featured-flavor",
            Panel {
                title: "Featured flavor",
                actions: rsx! {
                    span { class: "ff-name", "{name}" }
                    if let Some(price) = usd_price.as_ref() {
                        span { class: "ff-price", "{price}" }
                    }
                    if has_image {
                        button {
                            class: "panel-action",
                            onclick: move |_| overlay.set(Some(sd.clone())),
                            "Image"
                        }
                    }
                    if usd_price.is_some() {
                        a {
                            class: "panel-action",
                            href: "{tcgplayer}",
                            target: "_blank",
                            rel: "noopener",
                            "TCGplayer"
                            span { class: "ext", "\u{2197}" }
                        }
                        a {
                            class: "panel-action",
                            href: "{cardkingdom}",
                            target: "_blank",
                            rel: "noopener",
                            "Card Kingdom"
                            span { class: "ext", "\u{2197}" }
                        }
                    }
                },
                div { class: "ff-quote", "{flavor_text}" }
            }
        }
    }
}
