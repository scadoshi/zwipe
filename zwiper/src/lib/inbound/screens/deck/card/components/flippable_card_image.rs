//! Card image with a built-in flip control for double-faced cards.

use dioxus::prelude::*;
use std::{
    collections::HashSet,
    sync::{Mutex, OnceLock},
};
use zwipe_core::domain::card::scryfall_data::{ImageSize, ScryfallData};

/// Image URLs whose bytes have already arrived once this session. These
/// render instantly; only first-time loads play the ease-in. Keyed by URL
/// (not component) because stack shifts recreate component instances, which
/// would otherwise replay the fade on every already-cached image.
fn seen_urls() -> &'static Mutex<HashSet<String>> {
    static SEEN: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    SEEN.get_or_init(|| Mutex::new(HashSet::new()))
}

/// Forgets all seen URLs so every image's next render eases in again. Called
/// on deliberate stack refreshes — the fade doubles as feedback that fresh
/// results arrived.
pub(crate) fn reset_image_ease() {
    if let Ok(mut seen) = seen_urls().lock() {
        seen.clear();
    }
}

/// Renders a card image with a flip-icon overlay for cards that have multiple faces.
///
/// Owns the `face_idx` state internally — when a parent re-renders this component
/// with a different `sd`, Dioxus destroys/recreates the component and the face
/// state resets to 0 (the front face), which is the desired default.
///
/// Single-faced cards render just the image; the flip control is hidden so
/// non-DFC cards look identical to before.
#[component]
pub(crate) fn FlippableCardImage(
    sd: ReadSignal<ScryfallData>,
    size: ImageSize,
    #[props(default = String::new())] class: String,
    #[props(default = true)] draggable: bool,
    /// Whether to render the flip button overlay. Set `false` on non-interactive
    /// surfaces like exiting swipe-stack cards or peeking-underneath cards where
    /// the button would be confusingly visible without being tappable.
    #[props(default = true)]
    flippable: bool,
) -> Element {
    let mut face_idx: Signal<usize> = use_signal(|| 0_usize);
    // Bumped by the img's load event so the seen-URL check below re-runs.
    let mut load_nudge: Signal<u32> = use_signal(|| 0);
    let _ = load_nudge();
    let total = sd.read().face_count();
    let alt = sd.read().name.clone();
    let image_url: Option<String> = sd
        .read()
        .face_image_url(face_idx(), size)
        .map(str::to_owned);

    let already_loaded = image_url.as_ref().is_some_and(|u| {
        seen_urls()
            .lock()
            .map(|seen| seen.contains(u))
            .unwrap_or(false)
    });

    let flippable_class = if flippable && total > 1 {
        " flippable"
    } else {
        ""
    };

    rsx! {
        div { class: "flippable-card-wrapper{flippable_class} {class}",
            if let Some(url) = image_url {
                img {
                    src: "{url}",
                    alt: "{alt}",
                    draggable,
                    class: if already_loaded { "img-loaded" } else { "" },
                    onload: move |_| {
                        if let Ok(mut seen) = seen_urls().lock() {
                            seen.insert(url.clone());
                        }
                        load_nudge += 1;
                    },
                }
            }
            if flippable && total > 1 {
                button {
                    class: "card-flip-button",
                    "aria-label": "Flip card",
                    onclick: move |e| {
                        e.stop_propagation();
                        face_idx.set((face_idx() + 1) % total);
                    },
                    onpointerdown: move |e| { e.stop_propagation(); },
                    onmousedown: move |e| { e.stop_propagation(); },
                    ontouchstart: move |e| { e.stop_propagation(); },
                    "Flip"
                }
            }
        }
    }
}
