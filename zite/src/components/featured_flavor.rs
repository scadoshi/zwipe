//! Featured flavor — the hour's shared flavor card, mirrored from the app.
//!
//! Same unauthed endpoint the app home screen reads (`featured_flavor_route`);
//! the server flips the pick at the top of every UTC hour, so the site shows
//! a living element that matches what app users see at the same moment.
//! Fetch pattern mirrors `StatsStrip`; on any error the section hides itself
//! rather than breaking the home page. Tap the card-name tag to open the
//! full-art overlay (same pattern as the shared-deck page's).

use crate::{API_BASE, components::sleep_ms};
use dioxus::prelude::*;
use zwipe_components::FlippableCardImage;
use zwipe_core::{
    domain::card::{Card, scryfall_data::ImageSize},
    http::paths::featured_flavor_route,
};

#[component]
pub fn FeaturedFlavor() -> Element {
    let card: Resource<Option<Card>> = use_resource(|| async {
        let url = format!("{}{}", API_BASE, featured_flavor_route());
        let res = reqwest::Client::new().get(&url).send().await.ok()?;
        if !res.status().is_success() {
            return None;
        }
        res.json::<Card>().await.ok()
    });
    let mut show_overlay = use_signal(|| false);
    let mut overlay_dismissing = use_signal(|| false);

    let value = card.read();
    let Some(Some(card)) = &*value else {
        return rsx! {};
    };
    let Some(flavor_text) = card.scryfall_data.flavor_text.clone() else {
        return rsx! {};
    };
    let name = card.scryfall_data.name.clone();
    let sd = card.scryfall_data.clone();
    let has_image = sd.primary_image_url(ImageSize::Normal).is_some();

    rsx! {
        hr { class: "hero-rule" }
        section { class: "featured-flavor",
            div { class: "ff-title", "Featured flavor" }
            div { class: "ff-quote", "{flavor_text}" }
            div { class: "ff-name-row",
                span {
                    class: if has_image { "ff-name ff-name-link" } else { "ff-name" },
                    onclick: move |_| {
                        if has_image {
                            show_overlay.set(true);
                        }
                    },
                    "{name}"
                }
            }
        }
        // Tap-to-open full-art overlay, mirroring the shared-deck page: a
        // dimmed backdrop with the card art, tap anywhere to dismiss (the
        // flip button stops propagation, so flipping never dismisses).
        if show_overlay() || overlay_dismissing() {
            div { class: "sd-image-overlay-backdrop" }
            div {
                class: if overlay_dismissing() { "sd-image-overlay dismissing" } else { "sd-image-overlay" },
                onclick: move |_| {
                    overlay_dismissing.set(true);
                    spawn(async move {
                        sleep_ms(200).await;
                        show_overlay.set(false);
                        overlay_dismissing.set(false);
                    });
                },
                if show_overlay() {
                    FlippableCardImage {
                        sd,
                        size: ImageSize::Large,
                        class: "sd-image-overlay-img".to_string(),
                        draggable: false,
                    }
                }
            }
        }
    }
}
