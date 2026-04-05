//! Reusable bottom sheet overlay component.

use dioxus::prelude::*;

/// A slide-up bottom sheet with backdrop, title, content slot, and close button.
#[component]
pub fn BottomSheet(open: Signal<bool>, title: String, children: Element) -> Element {
    rsx! {
        div {
            class: if open() { "modal-backdrop show" } else { "modal-backdrop" },
            onclick: move |_| open.set(false),
        }
        div {
            class: if open() { "bottom-sheet show" } else { "bottom-sheet" },
            div { class: "modal-header",
                span { class: "text-muted", style: "font-size: 1rem;", "{title}" }
            }
            div { class: "modal-content",
                div { class: "flex-col",
                    style: "gap: 0.5rem;",
                    {children}
                }
            }
            div { class: "util-bar",
                button {
                    class: "util-btn",
                    onclick: move |_| open.set(false),
                    "close"
                }
            }
        }
    }
}
