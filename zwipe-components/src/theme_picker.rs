//! Site-style theme picker: a [`NavDropdown`] of every allowed theme
//! (color-blind themes grouped in their own bottom section) plus a dark/light
//! mode toggle.
//!
//! This is the canonical copy for every site; the host passes its `Signal<ThemeConfig>` in,
//! so how the theme is provided (context, prop drilling) and applied (body
//! class, wrapper div) stays the host's business.

use dioxus::prelude::*;
use zwipe_core::domain::user::{
    models::theme::ThemeConfig,
    preferences::{ALLOWED_THEMES, COLORBLIND_THEMES, display_theme_name},
};

use crate::NavDropdown;

/// Theme dropdown + dark/light toggle. Every theme has both modes, so the
/// toggle is always shown.
#[component]
pub fn ThemePicker(
    theme: Signal<ThemeConfig>,
    /// What the picker shows as current (its label, the active row, the mode
    /// toggle) when that should lag behind `theme`, as during a wipe. Defaults
    /// to `theme`.
    #[props(default)]
    shown: Option<Signal<ThemeConfig>>,
    /// The menu's open state, when the host wants it: a nav with menus of its
    /// own passes one so it can keep a single menu open at a time. Defaults to
    /// the picker's own.
    #[props(default)]
    open: Option<Signal<bool>>,
) -> Element {
    let mut theme = theme;
    // Always taken, so the hook order holds whether or not the host passes one.
    let own_open = use_signal(|| false);
    let mut open = open.unwrap_or(own_open);
    let shown = shown.unwrap_or(theme);
    let current = shown.read().name.clone();
    let is_dark = shown.read().is_dark;
    let mode = if is_dark { "dark" } else { "light" };
    // ALLOWED_THEMES is already alphabetical; filtering preserves that order
    // for the main group and pulls the color-blind themes into a bottom section.
    let regular_themes = ALLOWED_THEMES
        .iter()
        .copied()
        .filter(|t| !COLORBLIND_THEMES.contains(t));
    let colorblind_themes = ALLOWED_THEMES
        .iter()
        .copied()
        .filter(|t| COLORBLIND_THEMES.contains(t));

    rsx! {
        // Its children glide aside when the label changes width (NAV_GLIDE_JS).
        div { class: "theme-switcher", "data-nav-glide": "true",
            NavDropdown {
                open,
                label: display_theme_name(&current),
                div { class: "nav-dropdown-label", "Themes" }
                for name in regular_themes {
                    button {
                        class: if current == *name { "nav-dropdown-item active" } else { "nav-dropdown-item" },
                        onclick: move |_| {
                            let dark = theme.read().is_dark;
                            theme.set(ThemeConfig {
                                name: name.to_string(),
                                is_dark: dark,
                            });
                            open.set(false);
                        },
                        span { "{display_theme_name(name)}" }
                        ThemeDots { name, mode }
                    }
                }
                div { class: "nav-dropdown-label", "Color blind" }
                for name in colorblind_themes {
                    button {
                        class: if current == *name { "nav-dropdown-item active" } else { "nav-dropdown-item" },
                        onclick: move |_| {
                            let dark = theme.read().is_dark;
                            theme.set(ThemeConfig {
                                name: name.to_string(),
                                is_dark: dark,
                            });
                            open.set(false);
                        },
                        span { "{display_theme_name(name)}" }
                        ThemeDots { name, mode }
                    }
                }
            }
            button {
                class: "mode-toggle",
                onclick: move |_| {
                    let current = theme.read().clone();
                    theme.set(ThemeConfig {
                        name: current.name,
                        is_dark: !current.is_dark,
                    });
                },
                if is_dark { "light" } else { "dark" }
            }
        }
    }
}

/// A theme's colors as a strip of dots: background, text, the three accents
/// and the error color. The strip carries the theme's own class, so the dots
/// read its CSS variables and the colors stay defined only in themes.css.
#[component]
fn ThemeDots(name: &'static str, mode: &'static str) -> Element {
    rsx! {
        span { class: "theme-swatches theme-{name}-{mode}",
            span { class: "theme-dot", style: "background:var(--palette-1)" }
            span { class: "theme-dot", style: "background:var(--palette-2)" }
            span { class: "theme-dot", style: "background:var(--palette-3)" }
            span { class: "theme-dot", style: "background:var(--palette-4)" }
            span { class: "theme-dot", style: "background:var(--palette-5)" }
            span { class: "theme-dot", style: "background:var(--palette-6)" }
        }
    }
}
