//! Card image with a built-in flip control for double-faced cards.
//!
//! Shared by zwiper (swipe stack, printing sheet, fullscreen preview) and zite
//! (shared-deck image overlay) so the flip affordance behaves identically
//! everywhere. Owns its `face_idx` internally; a host can seed the first face
//! shown via `initial_face` (e.g. to open an overlay already on the back side).

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
/// on deliberate stack refreshes - the fade doubles as feedback that fresh
/// results arrived.
pub fn reset_image_ease() {
    if let Ok(mut seen) = seen_urls().lock() {
        seen.clear();
    }
}

/// Renders a card image with a flip-icon overlay for cards that have multiple faces.
///
/// Owns the `face_idx` state internally (seeded from `initial_face`) - when a
/// parent re-renders this component with a different `sd`, Dioxus destroys and
/// recreates it and the face resets to `initial_face`.
///
/// Single-faced cards render just the image; the flip control is hidden so
/// non-DFC cards look identical to before.
#[component]
pub fn FlippableCardImage(
    sd: ReadSignal<ScryfallData>,
    size: ImageSize,
    #[props(default = String::new())] class: String,
    #[props(default = true)] draggable: bool,
    /// Whether to render the flip button overlay. Set `false` on non-interactive
    /// surfaces like exiting swipe-stack cards or peeking-underneath cards where
    /// the button would be confusingly visible without being tappable.
    #[props(default = true)]
    flippable: bool,
    /// Face to show first (0 = front). Lets a host open the image already flipped
    /// to the side the user was viewing elsewhere.
    #[props(default = 0)]
    initial_face: usize,
) -> Element {
    let mut face_idx: Signal<usize> = use_signal(move || initial_face);
    // Bumped by the img's load event so the seen-URL check below re-runs.
    let mut load_nudge: Signal<u32> = use_signal(|| 0);
    let _ = load_nudge();
    let total = sd.read().face_count();
    // Clamp so a stale/over-large `initial_face` still resolves to a real face.
    let cur = face_idx().min(total.saturating_sub(1));
    let alt = sd.read().name.clone();
    let image_url: Option<String> = sd.read().face_image_url(cur, size).map(str::to_owned);

    // Text-proxy fields for the no-image placeholder: a card that carries no art
    // still shows its identity + rules so the user can read it and swipe past.
    // Read only when there's no image, so an art card never pays to clone its
    // oracle text.
    let (mana_cost, type_line, oracle_text, pt) = if image_url.is_none() {
        let sd_read = sd.read();
        let pt = match (&sd_read.power, &sd_read.toughness, &sd_read.loyalty) {
            (Some(p), Some(t), _) => Some(format!("{p}/{t}")),
            (_, _, Some(l)) => Some(l.clone()),
            _ => None,
        };
        (
            sd_read.mana_cost.clone(),
            sd_read.type_line.clone(),
            sd_read.oracle_text.clone(),
            pt,
        )
    } else {
        (None, None, None, None)
    };

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
            // Inner box shrink-wraps the image so the flip button anchors to the
            // image's own top-right corner, not the letterboxed wrapper - no layout
            // shift between DFC and single-faced cards.
            div { class: "flip-face",
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
                } else {
                    // No art: draw a card-shaped text proxy (name, mana, type,
                    // rules) in the image's footprint instead of an empty box.
                    div { class: "no-image-card",
                        div { class: "nic-titlebar",
                            span { class: "nic-name", "{alt}" }
                            if let Some(mc) = mana_cost {
                                span { class: "nic-mana", "{mc}" }
                            }
                        }
                        if let Some(tl) = type_line {
                            div { class: "nic-type", "{tl}" }
                        }
                        div { class: "nic-text",
                            if let Some(ot) = oracle_text {
                                "{ot}"
                            } else {
                                span { class: "nic-empty", "No card text." }
                            }
                        }
                        if let Some(p) = pt {
                            div { class: "nic-pt", "{p}" }
                        }
                    }
                }
                if flippable && total > 1 {
                    button {
                        class: "card-flip-button",
                        "aria-label": "Flip card",
                        onclick: move |e| {
                            e.stop_propagation();
                            face_idx.set((cur + 1) % total);
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
}
