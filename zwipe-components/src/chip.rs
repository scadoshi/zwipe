//! Selectable chip button.
//!
//! The single source for the small toggle buttons used across filter screens,
//! import/export controls, deck fields, and the shared deck page. Renders the
//! shared `.chip` / `.chip.selected` styling so every chip looks and behaves
//! the same in `zwiper` and `zite`.

use dioxus::prelude::*;

/// A selectable chip. `selected` drives the highlighted state, `onclick` fires
/// on tap, and `children` is the label (text or nodes).
#[component]
pub fn Chip(selected: bool, onclick: EventHandler<MouseEvent>, children: Element) -> Element {
    rsx! {
        button {
            class: if selected { "chip selected" } else { "chip" },
            onclick: move |evt| onclick.call(evt),
            {children}
        }
    }
}
