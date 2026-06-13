//! Reusable bottom sheet overlay component.

use dioxus::prelude::*;

/// A slide-up bottom sheet with backdrop, title, content slot, and footer.
///
/// `footer` overrides the default single "Close" button (e.g. a Back/Save pair).
/// `on_dismiss` fires when the backdrop is tapped, before the sheet closes.
#[component]
pub fn BottomSheet(
    mut open: Signal<bool>,
    title: String,
    children: Element,
    footer: Option<Element>,
    on_dismiss: Option<EventHandler<()>>,
) -> Element {
    rsx! {
        div {
            class: if open() { "modal-backdrop show" } else { "modal-backdrop" },
            onclick: move |_| {
                if let Some(h) = on_dismiss { h.call(()); }
                open.set(false);
            },
        }
        div {
            class: if open() { "bottom-sheet show" } else { "bottom-sheet" },
            div { class: "modal-header",
                span { style: "font-size: 1rem; color: var(--accent-tertiary);", "{title}" }
            }
            div { class: "modal-content",
                div { class: "flex-col", style: "gap: 0.5rem;",
                    {children}
                }
            }
            div { class: "util-bar",
                if let Some(f) = footer {
                    {f}
                } else {
                    button {
                        class: "util-btn",
                        onclick: move |_| open.set(false),
                        "Close"
                    }
                }
            }
        }
    }
}
