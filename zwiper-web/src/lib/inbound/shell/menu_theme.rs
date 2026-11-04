//! The main menu's theme buttons, after the footer's icons: a palette that
//! opens the Themes sheet and the mode's sun or moon, which flips Dark/Light.
//!
//! Both save through Settings' own path (the sheet and the dark mode toggle
//! in `screens::config`), so the pair and the Theme row never disagree. The
//! sheet is the menu's descendant, so the menu stays painted and folds as
//! the sheet rises, the way the kit's `ThemePicker` folds a site's menu.

use components::{DockIcon, ModeIcon, ThemeConfig};
use dioxus::prelude::*;

use crate::inbound::screens::config::{ThemesSheet, use_toggle_dark_mode};

/// The signed-in menu's theme buttons, two footer icons: the palette opens
/// the Themes sheet and folds the menu, and the sun or moon names the mode
/// in use and flips it, saving at once.
#[component]
pub fn MenuTheme(mut open: Signal<bool>) -> Element {
    let theme: Signal<ThemeConfig> = use_context();
    let toggle = use_toggle_dark_mode();
    let mut sheet_open = use_signal(|| false);
    let is_dark = theme.read().is_dark;
    rsx! {
        button {
            class: "btn-action nav-icon",
            "aria-label": "Theme",
            onclick: move |_| {
                open.set(false);
                sheet_open.set(true);
            },
            DockIcon { path: components::icons::PALETTE }
        }
        button {
            class: "btn-action nav-icon",
            "aria-label": if is_dark { "Switch to light" } else { "Switch to dark" },
            onclick: move |_| toggle.call(()),
            ModeIcon { dark: is_dark }
        }
        // The kit's class keeps the folding menu painted while the sheet is up.
        div { class: "theme-picker-sheet",
            ThemesSheet { open: sheet_open }
        }
    }
}
