//! The nav's pip style toggle, beside the theme picker and in its grammar:
//! a `mode-toggle` pill naming the style in use, with a red pip drawn in
//! that style standing in for the mode icon.

use dioxus::prelude::*;
use zwipe_core::domain::user::preferences::PipColor;

/// Flips the site's pip style between the theme's colors and the classic
/// ones. It names the style in use; the tooltip names the one a click
/// switches to.
#[component]
pub fn PipToggle(style: Signal<PipColor>) -> Element {
    let mut style = style;
    let current = style();
    rsx! {
        button {
            class: "mode-toggle pip-toggle",
            title: match current {
                PipColor::Theme => "Switch to classic pips",
                PipColor::Classic => "Switch to theme pips",
            },
            onclick: move |_| style.set(current.toggled()),
            span { class: "mode-label",
                i { class: "ms ms-r ms-cost", "aria-hidden": "true" }
                match current {
                    PipColor::Theme => "Theme",
                    PipColor::Classic => "Classic",
                }
            }
        }
    }
}
