//! Footer action bar: the row of buttons pinned at the bottom of a screen.
//! The CSS class is `.util-bar`, styled in `assets/components.css`.

use dioxus::prelude::*;

/// A horizontal bar of actions. `class` appends extra classes to `util-bar`.
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
