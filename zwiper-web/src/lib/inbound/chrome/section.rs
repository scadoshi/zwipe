//! The app's collapsible section: a titled card whose body eases open and
//! shut, one open at a time through a shared signal.
//!
//! Inside an element with the `web-sections-open` class, a wide window shows
//! every section open, with no arrow and the header's `?` always shown;
//! below the breakpoint the accordion works as in the app.

use components::is_press;
use dioxus::prelude::*;

use crate::inbound::chrome::media::use_wide;

/// One section of an accordion. `open_section` holds the title of the open
/// one; a click on the card toggles it, and so do Enter and Space on its
/// title. `header_accessory` (a `?`) shows only while open and never toggles
/// the section. `open_when_wide` marks a section inside `web-sections-open`,
/// which on a wide window is always open and leaves the tab order alone.
#[component]
pub fn CollapsibleSection(
    title: String,
    mut open_section: Signal<Option<String>>,
    #[props(default)] badge: Option<String>,
    #[props(default)] warn: bool,
    #[props(default)] header_accessory: Option<Element>,
    #[props(default)] open_when_wide: bool,
    children: Element,
) -> Element {
    let id = title.clone();
    let id_for_keys = title.clone();
    let wide = use_wide();
    let always_open = open_when_wide && wide();
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
            // Named for the settle, so a section that comes or goes eases the ones under it.
            "data-settle-key": "section-{id}",
            onclick: move |_| {
                if is_open {
                    open_section.set(None);
                } else {
                    open_section.set(Some(id.clone()));
                }
            },
            div { class: "web-section-head",
                div {
                    class: "web-section-title",
                    role: (!always_open).then_some("button"),
                    tabindex: (!always_open).then_some("0"),
                    "aria-expanded": (!always_open).then(|| is_open.to_string()),
                    onkeydown: move |evt: KeyboardEvent| {
                        if !always_open && is_press(&evt) {
                            if is_open {
                                open_section.set(None);
                            } else {
                                open_section.set(Some(id_for_keys.clone()));
                            }
                        }
                    },
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
            // A shut body is inert, so Tab and screen readers pass over it.
            div {
                class: if is_open { "collapsible open" } else { "collapsible" },
                inert: (!is_open && !always_open).then_some(true),
                onclick: move |evt| evt.stop_propagation(),
                div { class: "collapsible-inner",
                    hr { class: "web-section-rule" }
                    div { class: "web-section-body", {children} }
                }
            }
        }
    }
}
