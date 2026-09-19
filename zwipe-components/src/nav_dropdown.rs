//! Pill trigger that opens a floating menu, with a click-away backdrop and a
//! hover bridge so the menu survives the cursor's trip from trigger to menu.
//! Inside [`NavBar`](crate::NavBar)'s collapsed panel the menu pins to the
//! sticky nav instead, so it escapes the panel's overflow clip.

use dioxus::prelude::*;

/// Dropdown menu behind a nav pill trigger.
#[component]
pub fn NavDropdown(
    /// Open state. Close it from your item `onclick`s.
    open: Signal<bool>,
    /// Trigger text; the `▾` is appended.
    label: String,
    /// `.nav-dropdown-item` rows, optionally under `.nav-dropdown-label` groups.
    children: Element,
) -> Element {
    let mut open = open;
    let dropdown_class = if open() {
        "nav-dropdown nav-dropdown-open"
    } else {
        "nav-dropdown"
    };

    rsx! {
        if open() {
            div {
                class: "nav-dropdown-backdrop",
                onclick: move |_| open.set(false),
            }
        }
        div { class: "{dropdown_class}",
            button {
                class: "nav-dropdown-trigger",
                aria_expanded: "{open()}",
                onclick: move |evt| {
                    evt.stop_propagation();
                    let next = !open();
                    open.set(next);
                },
                "{label} ▾"
            }
            div { class: "nav-dropdown-content", {children} }
        }
    }
}
