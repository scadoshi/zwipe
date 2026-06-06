use super::flippable_card_image::FlippableCardImage;
use dioxus::prelude::*;
use std::time::Duration;
use tokio::time::sleep;
use zwipe_core::domain::card::scryfall_data::{ImageSize, ScryfallData};

/// Fullscreen image preview overlay with dismiss animation.
#[component]
pub(crate) fn ImagePreview(
    mut card: Signal<Option<ScryfallData>>,
    mut dismissing: Signal<bool>,
) -> Element {
    rsx! {
        if card().is_some() || dismissing() {
            div { class: "modal-backdrop show" }
            div {
                class: if dismissing() {
                    "image-preview-container show dismissing"
                } else {
                    "image-preview-container show"
                },
                onclick: move |_| {
                    dismissing.set(true);
                    spawn(async move {
                        sleep(Duration::from_millis(200)).await;
                        card.set(None);
                        dismissing.set(false);
                    });
                },
                if let Some(sd) = card() {
                    FlippableCardImage {
                        sd,
                        size: ImageSize::Large,
                        class: "card-image".to_string(),
                    }
                }
            }
        }
    }
}
