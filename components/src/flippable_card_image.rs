//! Card image with a built-in flip control for double-faced cards.
//!
//! Shared by zwiper (swipe stack, printing sheet, fullscreen preview) and zite
//! (shared-deck image overlay) so the flip affordance behaves identically
//! everywhere. Owns its `face_idx` internally; a host can seed the first face
//! shown via `initial_face` (e.g. to open an overlay already on the back side),
//! or hold the face itself through `face` to flip the card from outside, such
//! as from a key.

use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

use dioxus::prelude::*;
use zwipe_core::domain::card::card_profile::CardProfile;
use zwipe_core::domain::card::scryfall_data::{ImageSize, ScryfallData};

use crate::meld::use_meld;
use crate::{CardProxy, data_action};

/// Image URLs whose bytes have already arrived once this session. These
/// render instantly; only first-time loads play the ease-in. Keyed by URL
/// (not component) because stack shifts recreate component instances, which
/// would otherwise replay the fade on every already-cached image.
fn seen_urls() -> &'static Mutex<HashSet<String>> {
    static SEEN: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    SEEN.get_or_init(|| Mutex::new(HashSet::new()))
}

/// Whether the image at `url` has already arrived once this session.
pub(crate) fn image_seen(url: &str) -> bool {
    seen_urls().lock().is_ok_and(|seen| seen.contains(url))
}

/// Records that the image at `url` has arrived.
pub(crate) fn mark_image_seen(url: &str) {
    if let Ok(mut seen) = seen_urls().lock() {
        seen.insert(url.to_owned());
    }
}

/// Waits for the images showing `url` to decode, so they paint on the next
/// frame once shown. Returns at once where the browser can't say, or when a
/// decode fails.
pub(crate) async fn decoded(url: &str) {
    let mut decode = document::eval(DECODE_JS);
    if decode.send(url).is_ok() {
        let _settled: Result<bool, _> = decode.recv().await;
    }
}

/// Takes an image URL, awaits the decode of every image showing it, where
/// the browser has one, then answers.
const DECODE_JS: &str = r#"
const url = await dioxus.recv();
const imgs = [...document.querySelectorAll("img")].filter((img) => img.src === url && img.decode);
await Promise.allSettled(imgs.map((img) => img.decode()));
dioxus.send(true);
"#;

/// Forgets all seen URLs so every image's next render eases in again. Called
/// on deliberate stack refreshes, where the fade doubles as feedback that fresh
/// results arrived.
pub fn reset_image_ease() {
    if let Ok(mut seen) = seen_urls().lock() {
        seen.clear();
    }
}

/// A transparent image the size Scryfall serves a card at `size`, which a
/// no-art proxy sits over so the host's image rules give it the image's box.
fn sizer(size: ImageSize) -> &'static str {
    match size {
        ImageSize::Small => {
            "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='146' height='204'/%3E"
        }
        ImageSize::Normal => {
            "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='488' height='680'/%3E"
        }
        ImageSize::Large => {
            "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='672' height='936'/%3E"
        }
    }
}

/// Renders a card image with a flip-icon overlay for cards that have multiple faces.
///
/// Owns the `face_idx` state internally (seeded from `initial_face`), unless
/// the host passes `face`. When a parent re-renders this component with a
/// different `sd`, Dioxus destroys and recreates it and an internal face
/// resets to `initial_face`.
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
    /// Fires with the new face index on flip, so a host can mirror the shown
    /// side (e.g. open its image overlay on the face being viewed).
    #[props(default)]
    on_face_change: Option<EventHandler<usize>>,
    /// The face shown, held by the host: the flip control writes the next
    /// face here, and the host may set it too. `None` keeps the face inside
    /// the component, seeded from `initial_face`. An index past the last face
    /// shows the last.
    #[props(default)]
    face: Option<Signal<usize>>,
    /// Fires once, when the first face shown has its bytes and is decoded (or
    /// failed, or has no art), so a host can hold an entrance until every
    /// image is real.
    #[props(default)]
    on_load: Option<EventHandler<()>>,
    /// Whether an image already loaded once this session shows at once, as
    /// arrived. A host that must not show an image before this element has
    /// its bytes (a swipe stack, whose top card would otherwise draw while
    /// still downloading) passes `false`, so the image waits for its own
    /// load.
    #[props(default = true)]
    trust_seen: bool,
    /// The card's roles and tags, which a no-art proxy lists as an expanded
    /// row does. Left out, the proxy shows no roles.
    #[props(default)]
    profile: Option<CardProfile>,
) -> Element {
    let own_face: Signal<usize> = use_signal(move || initial_face);
    let mut face_idx = face.unwrap_or(own_face);
    let mut announced: Signal<bool> = use_signal(|| false);
    let mut announce = move || {
        if !announced() {
            announced.set(true);
            if let Some(handler) = on_load {
                handler.call(());
            }
        }
    };
    // The image URL this element has loaded and decoded itself.
    let loaded_here: Signal<Option<String>> = use_signal(|| None);
    // The image URL that failed to load here, which then draws the text proxy:
    // a failed image would otherwise leave the card see-through.
    let failed_here: Signal<Option<String>> = use_signal(|| None);
    // A meld half can show its melded card in its place.
    let meld = use_meld(&sd.read());
    let sd_now = sd.read();
    let view: &ScryfallData = meld.shown().unwrap_or(&sd_now);
    let total = view.face_count();
    // Clamp so a stale/over-large `initial_face` still resolves to a real face.
    let cur = face_idx().min(total.saturating_sub(1));
    let alt = view
        .card_faces
        .as_ref()
        .and_then(|f| f.get(cur))
        .map_or_else(|| view.name.clone(), |f| f.name.clone());
    let image_url: Option<String> = view
        .face_image_url(cur, size)
        .map(str::to_owned)
        .filter(|url| failed_here.read().as_deref() != Some(url.as_str()));
    let is_no_image = image_url.is_none();

    // The no-art proxy's card, the shown face of what shows (a melded card
    // while Meld is on).
    let proxy_sd = is_no_image.then(|| view.clone());
    drop(sd_now);

    let already_loaded = image_url
        .as_deref()
        .is_some_and(|u| loaded_here.read().as_deref() == Some(u) || (trust_seen && image_seen(u)));

    // A cached image or a no-art card is settled before any load event.
    let settled = already_loaded || is_no_image;
    use_effect(use_reactive!(|settled| {
        if settled {
            announce();
        }
    }));

    let flippable_class = if flippable && total > 1 {
        " flippable"
    } else {
        ""
    };

    rsx! {
        div { class: "flippable-card-wrapper{flippable_class} {class}",
            // Inner box shrink-wraps the image so the flip button anchors to the
            // image's own top-right corner, not the letterboxed wrapper, so there is
            // no layout shift between DFC and single-faced cards.
            div { class: "flip-face",
                if let Some(url) = image_url {
                    img {
                        src: "{url}",
                        alt: "{alt}",
                        draggable,
                        class: if already_loaded { "img-loaded" } else { "" },
                        // Shown only once decoded, so it never paints half
                        // drawn or a frame late.
                        onload: {
                            let url = url.clone();
                            move |_| {
                                let url = url.clone();
                                let mut loaded_here = loaded_here;
                                let mut announce = announce;
                                spawn(async move {
                                    decoded(&url).await;
                                    mark_image_seen(&url);
                                    loaded_here.set(Some(url.clone()));
                                    announce();
                                });
                            }
                        },
                        onerror: {
                            move |_| {
                                let mut failed_here = failed_here;
                                failed_here.set(Some(url.clone()));
                                announce();
                            }
                        },
                    }
                } else {
                    // No art, or art that failed: the card's text in the box the
                    // image would fill, Flip in its head for a multi-faced card.
                    // A transparent stand-in with the image's own shape and size,
                    // so every host's image rules size the proxy's box exactly as
                    // they would size the image.
                    if let Some(sd) = proxy_sd {
                        div { class: "card-proxy-slot",
                        img { class: "card-proxy-sizer", src: "{sizer(size)}", alt: "", draggable: false }
                        CardProxy {
                            sd,
                            face: cur,
                            profile,
                            trailing: (flippable && total > 1).then(|| rsx! {
                                button {
                                    class: "card-action-btn",
                                    "aria-label": "Flip card",
                                    "data-action": data_action::FLIP,
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
                            }),
                        }
                        }
                    }
                }
                if flippable && meld.available() && !is_no_image {
                    button {
                        class: if meld.on() { "card-flip-button card-meld-button selected" } else { "card-flip-button card-meld-button" },
                        "data-action": data_action::MELD,
                        "aria-label": if meld.on() { "Show this half" } else { "Show the melded card" },
                        "aria-pressed": "{meld.on()}",
                        onclick: move |e: MouseEvent| {
                            e.stop_propagation();
                            meld.toggle();
                        },
                        onpointerdown: move |e| { e.stop_propagation(); },
                        onmousedown: move |e| { e.stop_propagation(); },
                        ontouchstart: move |e| { e.stop_propagation(); },
                        "Meld"
                    }
                }
                if flippable && total > 1 && !is_no_image {
                    button {
                        class: "card-flip-button",
                        "aria-label": "Flip card",
                        "data-action": data_action::FLIP,
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
