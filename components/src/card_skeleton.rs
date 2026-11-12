//! A swipe card's ghost while cards load and where the stack has run out.

use dioxus::prelude::*;

/// A swipe card's ghost: the image block, while cards load and where the
/// stack has run out. `note` says why a filter left nothing to deal, and
/// `detail`, under it, whether Synergy may be the reason.
#[component]
pub fn CardSkeleton(
    #[props(default)] note: Option<&'static str>,
    #[props(default)] detail: Option<&'static str>,
) -> Element {
    rsx! {
        div { class: "skeleton-card",
            // The stack's slot, the card's ghost centered in it as the real
            // card is.
            div { class: "skeleton-stack",
                if let Some(note) = note {
                    div { class: "skeleton-image skeleton-note",
                        span { {note} }
                        if let Some(detail) = detail {
                            span { class: "skeleton-note-detail", {detail} }
                        }
                    }
                } else {
                    div { class: "skeleton-image" }
                }
            }
        }
    }
}
