//! The nav's pip style toggle, beside the theme picker and in its grammar:
//! a `mode-toggle` pill naming the style in use, with a red pip drawn in
//! that style standing in for the mode icon.

use dioxus::prelude::*;
use zwipe_core::domain::user::preferences::PipStyle;

/// Flips the site's pip style between the theme's colors and the classic
/// ones. It names the style in use; the tooltip names the one a click
/// switches to.
#[component]
pub fn PipToggle(style: Signal<PipStyle>) -> Element {
    let mut style = style;
    let current = style();
    rsx! {
        button {
            class: "mode-toggle pip-toggle",
            title: match current {
                PipStyle::Theme => "Switch to natural pips",
                PipStyle::Natural => "Switch to theme pips",
            },
            onclick: move |_| style.set(current.toggled()),
            span { class: "mode-label",
                i { class: "ms ms-r ms-cost", "aria-hidden": "true" }
                match current {
                    PipStyle::Theme => "Theme",
                    PipStyle::Natural => "Natural",
                }
            }
        }
    }
}
