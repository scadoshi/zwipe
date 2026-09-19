//! The small toggle button used across filter screens, deck fields, and the
//! shared deck page.

use dioxus::prelude::*;

/// A selectable chip; `children` is the label.
#[component]
pub fn Chip(
    selected: bool,
    onclick: EventHandler<MouseEvent>,
    /// Grayed out and inert.
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
