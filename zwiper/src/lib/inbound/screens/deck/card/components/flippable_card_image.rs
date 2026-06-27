//! Card image with a built-in flip control for double-faced cards.

use dioxus::prelude::*;
use zwipe_core::domain::card::scryfall_data::{ImageSize, ScryfallData};

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
    let total = sd.read().face_count();
    let alt = sd.read().name.clone();
    let image_url: Option<String> = sd
        .read()
        .face_image_url(face_idx(), size)
        .map(str::to_owned);

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
