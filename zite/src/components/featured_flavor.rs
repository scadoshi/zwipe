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
//! Image and the cart, which opens the store dialog for `TCGplayer` and Card
//! Kingdom. Image requests the full-art
//! overlay via the `overlay` signal; the HOME PAGE renders the overlay itself
//! at its top level, outside the `content-enter` content tree: nested here, an
//! animated ancestor's transform would
//! become the containing block for `position: fixed` and trap the overlay in
//! the page column.

use components::{CartButton, ImageButton, Panel, StoreDialog};
use dioxus::prelude::*;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::ScryfallData;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::deck_metrics;
use zwipe_ui::buy;

use crate::components::{OVERLAY_EXIT_MS, sleep_ms};

/// A real card response (Research Assistant, M15), decoded as the DEBUG-ONLY
/// fallback while the live fetch is pending or failing, so `dx serve` with no
/// backend still renders the element. Release builds hide the section until
/// live data lands (`StatsStrip` parity): a stale but real card looks exactly
/// like a live one.
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
        sleep_ms(OVERLAY_EXIT_MS).await;
        overlay.set(None);
        dismissing.set(false);
    });
}

#[component]
pub fn FeaturedFlavor(overlay: Signal<Option<ScryfallData>>) -> Element {
    let fetched: Resource<Option<Card>> =
        use_resource(|| async { crate::api::client().featured_flavor().await.ok() });
    let mut buy_open = use_signal(|| false);

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
    let usd_price = deck_metrics::card_price(&sd, PriceCurrency::Usd)
        .map(|p| PriceCurrency::Usd.format_amount(p));
    let links = buy::card_links(&name, sd.tcgplayer_id);

    rsx! {
        section { class: "featured-flavor",
            Panel {
                title: "Featured flavor",
                actions: rsx! {
                    span { class: "ff-name", "{name}" }
                    if let Some(price) = usd_price.as_ref() {
                        span { class: "ff-price", "{price}" }
                    }
                    // A card without art, or one whose art fails, opens on its
                    // name and text, as in the app.
                    ImageButton { on_click: move |()| overlay.set(Some(sd.clone())) }
                    if usd_price.is_some() {
                        CartButton { onclick: move |_| buy_open.set(true) }
                    }
                },
                div { class: "ff-quote", "{flavor_text}" }
            }
        }
        StoreDialog {
            open: buy_open,
            title: buy::CARD_TITLE.to_owned(),
            body: buy::CARD_BODY.to_owned(),
            links,
        }
    }
}
