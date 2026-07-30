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
pub fn Chip(
    selected: bool,
    onclick: EventHandler<MouseEvent>,
    /// Grayed out and inert — for a toggle whose target is currently empty
    /// or otherwise inapplicable. Off by default.
    #[props(default)]
    disabled: bool,
    children: Element,
) -> Element {
    let class = match (selected, disabled) {
        (_, true) => "chip disabled",
        (true, false) => "chip selected",
        (false, false) => "chip",
    };
    rsx! {
        button {
            class: "{class}",
            disabled,
            onclick: move |evt| {
                if !disabled {
                    onclick.call(evt);
                }
            },
            {children}
        }
    }
}
