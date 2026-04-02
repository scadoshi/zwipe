use dioxus::prelude::*;
use std::time::Duration;
use tokio::time::sleep;

/// Fullscreen image preview overlay with dismiss animation.
#[component]
pub(crate) fn ImagePreview(
    mut url: Signal<Option<String>>,
    mut dismissing: Signal<bool>,
) -> Element {
    rsx! {
        if url().is_some() || dismissing() {
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
                        url.set(None);
                        dismissing.set(false);
                    });
                },
                if let Some(url) = url() {
                    img {
                        src: "{url}",
                        alt: "card preview",
                        class: "card-image",
                    }
                }
            }
        }
    }
}
