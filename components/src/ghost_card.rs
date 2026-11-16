//! A swipe card's ghost: a card-shaped blank in a stack card of its own, so the card's own rules
//! give it the top card's exact box at every width, under a stack whose top card's image is
//! still on its way, or in a stack of its own while a pile loads or where it has run out.

use dioxus::prelude::*;

/// A blank in a card image's shape (Scryfall's 672 by 936), in the place a card's image shows,
/// so it sizes as the card's image does.
const CARD_GHOST_SRC: &str =
    "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='672' height='936'/%3E";

/// The top card's ghost: a stack card holding a card-shaped blank in the card's own wrapper,
/// drawn in a `.swipe-stack`. `note` says why there is no card (a filter left nothing to deal)
/// and `detail`, under it, what may be the reason, inside the card's box.
#[component]
pub fn GhostCard(
    #[props(default)] note: Option<String>,
    #[props(default)] detail: Option<String>,
) -> Element {
    rsx! {
        div { class: "swipe-stack-card swipe-stack-ghost-card", "aria-hidden": "true",
            div { class: "flippable-card-wrapper card-image",
                div { class: "flip-face",
                    img { class: "img-loaded card-ghost", src: CARD_GHOST_SRC, alt: "", draggable: false }
                    if let Some(note) = note {
                        div { class: "card-ghost-note",
                            span { {note} }
                            if let Some(detail) = detail {
                                span { class: "skeleton-note-detail", {detail} }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// The ghost of a swipe stack's top card where no stack draws: while a pile is on its way, or
/// where it has run out (`note` and `detail` say why). It is [`GhostCard`] in a stack of its own,
/// so it takes the top card's box exactly and the card lands where its ghost was.
#[component]
pub fn CardGhost(
    #[props(default)] note: Option<String>,
    #[props(default)] detail: Option<String>,
) -> Element {
    rsx! {
        div { class: "swipe-stack web-swipe-stack card-ghost-stack", "aria-hidden": "true",
            GhostCard { note, detail }
        }
    }
}
