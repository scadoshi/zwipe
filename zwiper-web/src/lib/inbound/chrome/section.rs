//! The app's collapsible section: a titled card whose body eases open and
//! shut, one open at a time through a shared signal.
//!
//! Inside an element with the `web-sections-open` class, a wide window shows
//! every section open, with no arrow and the header's `?` always shown;
//! below the breakpoint the accordion works as in the app.

use dioxus::prelude::*;

/// One section of an accordion. `open_section` holds the title of the open
/// one; a click on the card toggles it. `header_accessory` (a `?`) shows
/// only while open and never toggles the section.
#[component]
pub fn CollapsibleSection(
    title: String,
    mut open_section: Signal<Option<String>>,
    #[props(default)] badge: Option<String>,
    #[props(default)] warn: bool,
    #[props(default)] header_accessory: Option<Element>,
    children: Element,
) -> Element {
    let id = title.clone();
    let is_open = open_section() == Some(id.clone());
    let class = match (is_open, warn) {
        (true, true) => "web-section open warn",
        (true, false) => "web-section open",
        (false, true) => "web-section warn",
        (false, false) => "web-section",
    };
    rsx! {
        div {
            class,
            onclick: move |_| {
                if is_open {
                    open_section.set(None);
                } else {
                    open_section.set(Some(id.clone()));
                }
            },
            div { class: "web-section-head",
                div { class: "web-section-title",
                    span { class: "card-row-arrow web-section-arrow", "\u{25b8}" }
                    span { class: "settings-list-title", "{title}" }
                    if let Some(badge) = badge.as_ref() {
                        span { class: "web-section-badge", "{badge}" }
                    }
                }
                if let Some(accessory) = header_accessory {
                    div {
                        class: "web-section-accessory",
                        onclick: move |evt| evt.stop_propagation(),
                        {accessory}
                    }
                }
            }
            div {
                class: if is_open { "collapsible open" } else { "collapsible" },
                onclick: move |evt| evt.stop_propagation(),
                div { class: "collapsible-inner",
                    hr { class: "web-section-rule" }
                    div { class: "web-section-body", {children} }
                }
            }
        }
    }
}
