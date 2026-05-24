//! User preferences screen for theme and dark mode selection.

use crate::{
    inbound::components::auth::{bouncer::Bouncer, session_upkeep::Upkeep},
    outbound::client::{user::preferences::ClientUpdatePreferences, ZwipeClient},
};
use zwipe_core::domain::user::models::theme::ThemeConfig;
use dioxus::prelude::*;
use dioxus_primitives::toast::{use_toast, ToastOptions};
use std::time::Duration;
use zwipe_core::http::contracts::user::HttpUpdatePreferences;
use zwipe_core::domain::{
    auth::models::session::Session,
    user::preferences::ALLOWED_THEMES,
};

/// Capitalize each word of a theme slug for display ("tokyo-night" → "Tokyo Night").
fn display_theme_name(slug: &str) -> String {
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

/// Preferences screen for selecting theme and light/dark mode.
#[component]
pub fn Preferences() -> Element {
    let navigator = use_navigator();

    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let mut theme_config: Signal<ThemeConfig> = use_context();
    let toast = use_toast();

    // Snapshot the original theme so we can restore on back-without-save
    let original_theme = use_signal(|| theme_config.read().clone());

    let mut selected_theme = use_signal(|| theme_config.read().name.clone());
    let mut selected_dark = use_signal(|| theme_config.read().is_dark);
    let mut is_loading = use_signal(|| false);
    let mut saved = use_signal(|| false);

    // Live preview: update theme_config whenever selection changes
    let mut apply_preview = move || {
        theme_config.set(ThemeConfig {
            name: selected_theme(),
            is_dark: selected_dark(),
        });
    };

    let mut save = move || {
        is_loading.set(true);
        let dark_mode = selected_dark();
        let request = HttpUpdatePreferences {
            theme: Some(selected_theme()),
            dark_mode: Some(dark_mode),
        };
        spawn(async move {
            session.upkeep(client);
            let Some(session_val) = session() else {
                toast.error(
                    "Session expired — please log in again".to_string(),
                    ToastOptions::default().duration(Duration::from_millis(3000)),
                );
                is_loading.set(false);
                return;
            };
            match client().update_preferences(request, &session_val).await {
                Ok(prefs) => {
                    theme_config.set(ThemeConfig::from(&prefs));
                    saved.set(true);
                    toast.success(
                        "Preferences saved".to_string(),
                        ToastOptions::default().duration(Duration::from_millis(1500)),
                    );
                    is_loading.set(false);
                }
                Err(e) => {
                    tracing::warn!("update preferences failed: {e}");
                    toast.error(
                        e.to_user_message(),
                        ToastOptions::default().duration(Duration::from_millis(3000)),
                    );
                    is_loading.set(false);
                }
            }
        });
    };

    rsx! {
        Bouncer {
            div { class: "screen",
                div { class: "page-header",
                    h2 { "Preferences" }
                }

                div { class: "screen-content centered content-enter",
                    div { class: "container-sm",

                        for theme in ALLOWED_THEMES {
                            button {
                                class: if selected_theme() == *theme { "pref-row selected" } else { "pref-row" },
                                onclick: move |_| {
                                    selected_theme.set(theme.to_string());
                                    apply_preview();
                                },
                                { display_theme_name(theme) }
                            }
                        }

                        div {
                            class: "pref-toggle",
                            span { "Dark mode" }
                            button {
                                class: "pref-toggle-btn",
                                onclick: move |_| {
                                    selected_dark.set(!selected_dark());
                                    apply_preview();
                                },
                                if selected_dark() { "On" } else { "Off" }
                            }
                        }
                    }
                }

                div { class: "util-bar",
                    button {
                        class: "util-btn",
                        onclick: move |_| {
                            if !saved() {
                                theme_config.set(original_theme());
                            }
                            navigator.go_back();
                        },
                        "Back"
                    }
                    button {
                        class: "util-btn",
                        disabled: is_loading(),
                        onclick: move |_| save(),
                        if is_loading() { "Saving..." } else { "Save" }
                    }
                }
            }
        }
    }
}
