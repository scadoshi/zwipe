//! The title bar a form panel wears, the kit panel's head: the screen's
//! title over a full-bleed rule.

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
