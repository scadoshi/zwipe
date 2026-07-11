//! User preferences screen for theme and dark mode selection.

use crate::{
    inbound::components::{auth::ensure_session::EnsureFresh, bottom_sheet::BottomSheet},
    outbound::client::{ZwipeClient, user::preferences::ClientUpdatePreferences},
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::time::Duration;
use zwipe_components::{Button, ButtonVariant};
use zwipe_core::{
    domain::{
        auth::models::session::Session,
        user::{models::theme::ThemeConfig, preferences::ALLOWED_THEMES},
    },
    http::contracts::user::HttpUpdatePreferences,
};

/// Capitalize each word of a theme slug for display ("tokyo-night" → "Tokyo Night").
pub(crate) fn display_theme_name(slug: &str) -> String {
    if slug == "rose-pine" {
        return "Rosé Pine".to_string();
    }
    slug.split('-')
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Themes with adjusted palettes for color-vision deficiency — grouped at the
/// bottom of the picker.
const COLORBLIND_THEMES: &[&str] = &["protanopia", "deuteranopia", "tritanopia"];

/// One selectable theme row: name on the left, color-swatch dots on the right.
/// The dots pull their colors from the theme's own CSS variables by applying
/// that theme's class to the swatch strip, so colors stay defined only in
/// themes.css.
#[component]
fn ThemeRow(
    theme: String,
    mode: String,
    mut selected_theme: Signal<String>,
    mut theme_config: Signal<ThemeConfig>,
    selected_dark: Signal<bool>,
) -> Element {
    let is_selected = selected_theme() == theme;
    let click_theme = theme.clone();
    rsx! {
        button {
            class: if is_selected { "pref-row selected" } else { "pref-row" },
            onclick: move |_| {
                selected_theme.set(click_theme.clone());
                theme_config.set(ThemeConfig { name: click_theme.clone(), is_dark: selected_dark() });
            },
            div { class: "pref-row-inner",
                span { "{display_theme_name(&theme)}" }
                div { class: "theme-swatches theme-{theme}-{mode}",
                    span { class: "theme-dot", style: "background:var(--bg-primary)" }
                    span { class: "theme-dot", style: "background:var(--text-primary)" }
                    span { class: "theme-dot", style: "background:var(--accent-primary)" }
                    span { class: "theme-dot", style: "background:var(--accent-secondary)" }
                    span { class: "theme-dot", style: "background:var(--accent-tertiary)" }
                    span { class: "theme-dot", style: "background:var(--color-error)" }
                }
            }
        }
    }
}

/// Bottom sheet for selecting theme and light/dark mode. Selections live-preview
/// against the whole app; Save persists and drops the sheet, Back/backdrop
/// restores the theme that was active when the sheet opened.
#[component]
pub fn PreferencesSheet(mut open: Signal<bool>) -> Element {
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let mut theme_config: Signal<ThemeConfig> = use_context();
    let toast = use_toast();

    let mut original_theme = use_signal(|| theme_config.peek().clone());
    let mut selected_theme = use_signal(|| theme_config.peek().name.clone());
    let mut selected_dark = use_signal(|| theme_config.peek().is_dark);

    // Snapshot the active theme each time the sheet opens and sync the selection
    // to it, so every open starts from the live theme.
    use_effect(move || {
        if open() {
            let current = theme_config.peek().clone();
            original_theme.set(current.clone());
            selected_theme.set(current.name.clone());
            selected_dark.set(current.is_dark);
        }
    });

    let mut save = move || {
        let request = HttpUpdatePreferences {
            theme: Some(selected_theme()),
            dark_mode: Some(selected_dark()),
        };
        open.set(false);
        spawn(async move {
            let session_val = match session.ensure_fresh(client).await {
                Ok(session_val) => session_val,
                Err(e) => {
                    toast.error(
                        e.to_user_message(),
                        ToastOptions::default().duration(Duration::from_millis(3000)),
                    );
                    return;
                }
            };
            match client().update_preferences(request, &session_val).await {
                Ok(prefs) => {
                    theme_config.set(ThemeConfig::from(&prefs));
                    toast.success(
                        "Theme saved".to_string(),
                        ToastOptions::default().duration(Duration::from_millis(1500)),
                    );
                }
                Err(e) => {
                    tracing::warn!("update preferences failed: {e}");
                    toast.error(
                        e.to_user_message(),
                        ToastOptions::default().duration(Duration::from_millis(3000)),
                    );
                }
            }
        });
    };

    let mode = (if selected_dark() { "dark" } else { "light" }).to_string();
    let regular_themes = ALLOWED_THEMES
        .iter()
        .copied()
        .filter(|t| !COLORBLIND_THEMES.contains(t));
    let colorblind_themes = ALLOWED_THEMES
        .iter()
        .copied()
        .filter(|t| COLORBLIND_THEMES.contains(t));

    rsx! {
        BottomSheet {
            open,
            title: "Themes".to_string(),
            on_dismiss: move |_| { theme_config.set(original_theme()); },
            footer: rsx! {
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| {
                        theme_config.set(original_theme());
                        open.set(false);
                    },
                    "Back"
                }
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| save(),
                    "Save"
                }
            },

            for theme in regular_themes {
                ThemeRow {
                    theme: theme.to_string(),
                    mode: mode.clone(),
                    selected_theme,
                    theme_config,
                    selected_dark,
                }
            }

            div { class: "pref-section-label", "Color blind" }

            for theme in colorblind_themes {
                ThemeRow {
                    theme: theme.to_string(),
                    mode: mode.clone(),
                    selected_theme,
                    theme_config,
                    selected_dark,
                }
            }
        }
    }
}
