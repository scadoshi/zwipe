//! Footer action bar.
//!
//! Wraps the row of buttons pinned at the bottom of a screen (Back / Save /
//! Cancel and friends). Carries the good name at the usage layer; the CSS
//! class stays `.util-bar` so the shared styling in `assets/components.css`
//! applies without a churny rename across every call site.

use dioxus::prelude::*;

/// A horizontal bar of actions, typically pinned to the bottom of a screen.
///
/// `class` appends extra classes (e.g. an entry animation) alongside the base
/// `util-bar`.
#[component]
pub fn ActionBar(#[props(default)] class: Option<String>, children: Element) -> Element {
    let full = match &class {
        Some(extra) => format!("util-bar {extra}"),
        None => "util-bar".to_string(),
    };
    rsx! {
        div { class: "{full}", {children} }
    }
}
