//! Card image with a flip control for double-faced cards, shared by zwiper and
//! zite so the affordance behaves identically everywhere.

use dioxus::prelude::*;
use std::{
    collections::HashSet,
    sync::{Mutex, OnceLock},
};
use zwipe_core::domain::card::scryfall_data::{ImageSize, ScryfallData};

use crate::OracleText;

/// Image URLs already loaded this session; only first-time loads ease in.
/// Keyed by URL because stack shifts recreate component instances, which would
/// otherwise replay the fade on cached images.
fn seen_urls() -> &'static Mutex<HashSet<String>> {
    static SEEN: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    SEEN.get_or_init(|| Mutex::new(HashSet::new()))
}

/// Forgets all seen URLs so every image eases in again. The fade doubles as
/// feedback on a deliberate stack refresh.
pub fn reset_image_ease() {
    if let Ok(mut seen) = seen_urls().lock() {
        seen.clear();
    }
}

/// Card image with a flip overlay for multi-faced cards. Owns `face_idx`; a
/// new `sd` recreates the component and resets it to `initial_face`.
#[component]
pub fn FlippableCardImage(
    sd: ReadSignal<ScryfallData>,
    size: ImageSize,
    #[props(default = String::new())] class: String,
    #[props(default = true)] draggable: bool,
    /// Render the flip button. Off for non-interactive surfaces (exiting or
    /// peeking stack cards) where it would be visible but not tappable.
    #[props(default = true)]
    flippable: bool,
    /// Face to show first, 0 being the front.
    #[props(default = 0)]
    initial_face: usize,
    /// Fires with the new face index on flip.
    #[props(default)]
    on_face_change: Option<EventHandler<usize>>,
) -> Element {
    let mut face_idx: Signal<usize> = use_signal(move || initial_face);
    // Bumped by the img's load event so the seen-URL check re-runs.
    let mut load_nudge: Signal<u32> = use_signal(|| 0);
    let _ = load_nudge();
    let total = sd.read().face_count();
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

    // No-image placeholder fields, resolved for the shown face. Only read when
    // there's no image, so an art card pays nothing.
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
            // Shrink-wraps the image so the flip button anchors to its corner,
            // not the letterboxed wrapper.
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
                    // No art: a card-shaped text proxy in the image's footprint.
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
                                            let next = (cur + 1) % total;
                                            face_idx.set(next);
                                            if let Some(handler) = on_face_change {
                                                handler.call(next);
                                            }
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
                            let next = (cur + 1) % total;
                            face_idx.set(next);
                            if let Some(handler) = on_face_change {
                                handler.call(next);
                            }
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
