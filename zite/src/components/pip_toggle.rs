//! The nav's pip style toggle, beside the theme picker and in its grammar:
//! a `mode-toggle` pill labeled Pips, with a red pip drawn in the style in
//! use standing in for the mode icon.

use dioxus::prelude::*;
use zwipe_core::domain::user::settings::PipStyle;

/// Flips the site's pip style between the theme's colors and the classic
/// ones. Its pip shows the style in use; the tooltip names the one a click
/// switches to.
#[component]
pub fn PipToggle(style: Signal<PipStyle>) -> Element {
    let mut style = style;
    let current = style();
    rsx! {
        button {
            class: "mode-toggle pip-toggle",
            title: match current {
                PipStyle::Themed => "Switch to classic pips",
                PipStyle::Classic => "Switch to themed pips",
            },
            onclick: move |_| style.set(current.toggled()),
            span { class: "mode-label",
                i { class: "ms ms-r ms-cost", "aria-hidden": "true" }
                "Pips"
            }
        }
    }
}
