//! A [`NavDropdown`] of every allowed theme plus a dark/light toggle. The host
//! passes its `Signal<ThemeConfig>` in; how the theme is provided and applied
//! stays the host's business.

use dioxus::prelude::*;
use zwipe_core::domain::user::{models::theme::ThemeConfig, preferences::ALLOWED_THEMES};

use crate::NavDropdown;

/// Themes grouped in their own bottom section, matching zwiper's preferences sheet.
const COLORBLIND_THEMES: &[&str] = &["protanopia", "deuteranopia", "tritanopia", "achromatopsia"];

/// Title-cased label for a theme slug, with brand casings special-cased.
fn display_theme_name(slug: &str) -> String {
    match slug {
        "rose-pine" => return "Rosé Pine".to_string(),
        "vscode" => return "VS Code".to_string(),
        "github" => return "GitHub".to_string(),
        "synthwave-84" => return "Synthwave '84".to_string(),
        "powershell" => return "PowerShell".to_string(),
        "docs-rs" => return "docs.rs".to_string(),
        _ => {}
    }
    slug.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Theme dropdown + dark/light toggle. Every theme has both modes, so the
/// toggle is always shown.
#[component]
pub fn ThemePicker(theme: Signal<ThemeConfig>) -> Element {
    let mut theme = theme;
    let mut open = use_signal(|| false);
    let current = theme.read().name.clone();
    let is_dark = theme.read().is_dark;
    let regular_themes = ALLOWED_THEMES
        .iter()
        .copied()
        .filter(|t| !COLORBLIND_THEMES.contains(t));
    let colorblind_themes = ALLOWED_THEMES
        .iter()
        .copied()
        .filter(|t| COLORBLIND_THEMES.contains(t));

    rsx! {
        div { class: "theme-switcher",
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
                        "{display_theme_name(name)}"
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
                        "{display_theme_name(name)}"
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
