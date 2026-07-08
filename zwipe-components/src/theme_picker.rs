//! Site-style theme picker: a dropdown of every allowed theme (color-blind
//! themes grouped in their own bottom section) plus a dark/light mode toggle.
//!
//! Authored in zite and lifted here verbatim (the canonical copy per the
//! portfolio-adoption ruling); the host passes its `Signal<ThemeConfig>` in,
//! so how the theme is provided (context, prop drilling) and applied (body
//! class, wrapper div) stays the host's business.

use dioxus::prelude::*;
use zwipe_core::domain::user::{models::theme::ThemeConfig, preferences::ALLOWED_THEMES};

/// Themes shown in their own bottom section of the picker. Mirrors the app's
/// preferences sheet (zwiper) so every surface groups these identically.
const COLORBLIND_THEMES: &[&str] = &["protanopia", "deuteranopia", "tritanopia"];

/// Human-readable label for a theme slug: title-cased words, with the accents
/// that title-casing can't produce special-cased.
fn display_theme_name(slug: &str) -> String {
    if slug == "rose-pine" {
        return "Rosé Pine".to_string();
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
    let select_class = if open() {
        "theme-select theme-select-open"
    } else {
        "theme-select"
    };

    rsx! {
        if open() {
            div {
                class: "theme-backdrop",
                onclick: move |_| open.set(false),
            }
        }
        div { class: "theme-switcher",
            div { class: "{select_class}",
                button {
                    class: "theme-select-trigger",
                    aria_expanded: "{open()}",
                    onclick: move |evt| {
                        evt.stop_propagation();
                        let next = !open();
                        open.set(next);
                    },
                    "{display_theme_name(&current)} ▾"
                }
                div { class: "theme-select-content",
                    div { class: "theme-select-label", "Themes" }
                    for name in regular_themes {
                        button {
                            class: if current == *name { "theme-option active" } else { "theme-option" },
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
                    div { class: "theme-select-label", "Color blind" }
                    for name in colorblind_themes {
                        button {
                            class: if current == *name { "theme-option active" } else { "theme-option" },
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
