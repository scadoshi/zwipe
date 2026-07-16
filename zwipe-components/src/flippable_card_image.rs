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

use crate::OracleText;

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
    let alt = {
        let sd_read = sd.read();
        sd_read
            .card_faces
            .as_ref()
            .and_then(|f| f.get(cur))
            .map(|f| f.name.clone())
            .unwrap_or_else(|| sd_read.name.clone())
    };
    let image_url: Option<String> = sd.read().face_image_url(cur, size).map(str::to_owned);
    let is_no_image = image_url.is_none();

    // Identity fields for the no-image placeholder, resolved for the shown face
    // (so a flipped DFC shows the back's text, not the front's). Rarity/set are
    // card-level. Read only when there's no image, so an art card pays nothing.
    let (mana_cost, type_line, rarity, set_name, oracle_text, flavor_text, pt) = if is_no_image {
        let sd_read = sd.read();
        let face = sd_read.card_faces.as_ref().and_then(|f| f.get(cur));
        let mana = face
            .map(|f| f.mana_cost.clone())
            .unwrap_or_else(|| sd_read.mana_cost.clone().unwrap_or_default());
        let (power, toughness, loyalty) = match face {
            Some(f) => (f.power.clone(), f.toughness.clone(), f.loyalty.clone()),
            None => (
                sd_read.power.clone(),
                sd_read.toughness.clone(),
                sd_read.loyalty.clone(),
            ),
        };
        let pt = match (power, toughness) {
            (Some(p), Some(t)) => Some(format!("{p}/{t}")),
            _ => loyalty,
        };
        (
            Some(mana).filter(|m| !m.is_empty()),
            face.and_then(|f| f.type_line.clone())
                .or_else(|| sd_read.type_line.clone()),
            Some(sd_read.rarity.to_long_name()),
            Some(sd_read.set_name.clone()),
            face.and_then(|f| f.oracle_text.clone())
                .or_else(|| sd_read.oracle_text.clone()),
            face.and_then(|f| f.flavor_text.clone())
                .or_else(|| sd_read.flavor_text.clone()),
            pt,
        )
    } else {
        (None, None, None, None, None, None, None)
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
                        div { class: "nic-head",
                            span { class: "nic-name", "{alt}" }
                            div { class: "nic-head-right",
                                if flippable && total > 1 {
                                    button {
                                        class: "card-action-btn",
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
                                if let Some(mc) = mana_cost {
                                    OracleText { text: mc, class: "card-detail-cost".to_string() }
                                }
                            }
                        }
                        hr { class: "nic-rule" }
                        // Boxed marker where the art would be, so it's clear why
                        // this card is text-only.
                        div { class: "nic-art",
                            span { class: "nic-noart", "No image" }
                        }
                        div { class: "nic-meta",
                            if let Some(tl) = type_line {
                                span { class: "nic-chip nic-chip-type", "{tl}" }
                            }
                            if let Some(r) = rarity {
                                span { class: "nic-chip nic-chip-rarity", "{r}" }
                            }
                            if let Some(s) = set_name {
                                span { class: "nic-chip nic-chip-set", "{s}" }
                            }
                        }
                        hr { class: "nic-rule" }
                        div { class: "nic-textbox",
                            if let Some(ot) = oracle_text {
                                OracleText { text: ot, class: "nic-oracle".to_string() }
                            }
                            if let Some(fl) = flavor_text {
                                div { class: "nic-flavor", "{fl}" }
                            }
                        }
                        if let Some(p) = pt {
                            span { class: "nic-chip nic-pt", "{p}" }
                        }
                    }
                }
                if flippable && total > 1 && !is_no_image {
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
