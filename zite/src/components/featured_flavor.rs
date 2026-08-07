//! Featured flavor — the hour's shared flavor card, mirrored from the app.
//!
//! Same unauthed endpoint the app home screen reads (`featured_flavor_route`);
//! the server flips the pick at the top of every UTC hour, so the site shows
//! a living element that matches what app users see at the same moment.
//! Fetch pattern mirrors `StatsStrip`: hide until live data lands. Debug
//! builds additionally fall back to a frozen real response (below) so the
//! element always renders under `dx serve` with no backend.
//!
//! Tapping the card-name tag requests the full-art overlay via the `overlay`
//! signal; the HOME PAGE renders the overlay itself at its top level, outside
//! the `content-enter` content tree — mirroring the shared-deck page. Keeping
//! the overlay nested here was the trapped-in-the-page-column bug: an
//! animated ancestor's transform makes it the containing block for
//! `position: fixed`.

use crate::{API_BASE, components::sleep_ms};
use dioxus::prelude::*;
use zwipe_components::Panel;
use zwipe_core::{
    domain::card::{
        Card,
        scryfall_data::{ImageSize, ScryfallData},
    },
    http::paths::featured_flavor_route,
};

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
pub fn dismiss_flavor_overlay(
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

#[component]
pub fn FeaturedFlavor(overlay: Signal<Option<ScryfallData>>) -> Element {
    let fetched: Resource<Option<Card>> = use_resource(|| async {
        let url = format!("{}{}", API_BASE, featured_flavor_route());
        let res = reqwest::Client::new().get(&url).send().await.ok()?;
        if !res.status().is_success() {
            return None;
        }
        res.json::<Card>().await.ok()
    });

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
    let name = card.scryfall_data.name.clone();
    let sd = card.scryfall_data.clone();
    let has_image = sd.primary_image_url(ImageSize::Normal).is_some();

    rsx! {
        section { class: "featured-flavor",
            Panel { eyebrow: "Flavor", title: "Featured flavor",
                div { class: "ff-quote", "{flavor_text}" }
                div { class: "ff-name-row",
                    span {
                        class: if has_image { "ff-name ff-name-link" } else { "ff-name" },
                        onclick: move |_| {
                            if has_image {
                                overlay.set(Some(sd.clone()));
                            }
                        },
                        "{name}"
                    }
                }
            }
        }
    }
}
