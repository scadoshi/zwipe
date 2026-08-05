//! Featured flavor — the hour's shared flavor card, mirrored from the app.
//!
//! Same unauthed endpoint the app home screen reads (`featured_flavor_route`);
//! the server flips the pick at the top of every UTC hour, so the site shows
//! a living element that matches what app users see at the same moment.
//! Fetch pattern mirrors `StatsStrip`; on any error the section hides itself
//! rather than breaking the home page. Tap the card name to reveal the card.

use crate::API_BASE;
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
    let mut show_image = use_signal(|| false);

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
        section { class: "featured-flavor",
            div { class: "ff-title", "Featured flavor" }
            div { class: "ff-quote", "{flavor_text}" }
            div {
                class: if has_image { "ff-name ff-name-link" } else { "ff-name" },
                onclick: move |_| {
                    if has_image {
                        show_image.set(!show_image());
                    }
                },
                "{name}"
            }
            if show_image() {
                div { class: "ff-image",
                    FlippableCardImage {
                        sd,
                        size: ImageSize::Normal,
                        class: "ff-card-image".to_string(),
                        draggable: false,
                    }
                }
            }
        }
    }
}
