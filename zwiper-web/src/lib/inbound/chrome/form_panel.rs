//! The title bar and the action footer a form panel wears,
//! the kit panel's head and actions: the screen's title over a full-bleed
//! rule, and a full-bleed rule over the form's buttons.

use dioxus::prelude::*;

/// The head of a form panel or a panel beside one, titled after its screen
/// (or, beside a form, after what it holds), at every width.
#[component]
pub fn FormPanelHead(title: String) -> Element {
    rsx! {
        div { class: "panel-head web-panel-head",
            h3 { class: "panel-title", "{title}" }
        }
        hr { class: "panel-rule web-panel-head" }
    }
}

/// The footer of a form panel: a full-bleed rule, then the form's buttons
/// one under another at the panel's width, the primary first, as the kit
/// panel's actions, at every width.
#[component]
pub fn FormPanelActions(children: Element) -> Element {
    rsx! {
        hr { class: "panel-rule web-panel-head" }
        div { class: "panel-actions web-form-actions web-form-actions-stack", {children} }
    }
}
