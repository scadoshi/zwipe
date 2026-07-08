//! Nav dropdown: a pill trigger that opens a floating menu of plain rows,
//! with a transparent click-away backdrop and an invisible hover bridge so
//! the menu doesn't close while the cursor travels from trigger to menu.
//!
//! The host owns the `open` signal (so item `onclick`s can close the menu
//! after acting) and the menu contents — typically `.nav-dropdown-item`
//! buttons/links, optionally grouped under `.nav-dropdown-label` subtitles.
//! [`ThemePicker`] is built on this; a site nav can use it for its own menus
//! (e.g. a Projects dropdown).
//!
//! Inside [`NavBar`]'s collapsed panel the menu pins to the sticky nav
//! instead of the panel (see the `.nav-panel .nav-dropdown` rules), so it
//! escapes the panel's overflow clip.
//!
//! [`ThemePicker`]: crate::ThemePicker
//! [`NavBar`]: crate::NavBar

use dioxus::prelude::*;

/// Dropdown menu behind a nav pill trigger.
#[component]
pub fn NavDropdown(
    /// Open state, host-owned: close it from your item `onclick`s.
    open: Signal<bool>,
    /// Trigger text; the dropdown appends the `▾` indicator.
    label: String,
    /// Menu contents (`.nav-dropdown-item` rows, `.nav-dropdown-label` groups).
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
