//! User preferences screen for theme and dark mode selection.

use crate::{
    domain::theme::ThemeConfig,
    inbound::components::auth::{bouncer::Bouncer, session_upkeep::Upkeep},
    outbound::client::{user::preferences::ClientUpdatePreferences, ZwipeClient},
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{use_toast, ToastOptions};
use std::time::Duration;
use zwipe::{
    domain::{
        auth::models::session::Session,
        user::models::preferences::ALLOWED_THEMES,
    },
    inbound::http::handlers::user::update_preferences::HttpUpdatePreferences,
};

/// Preferences screen for selecting theme and light/dark mode.
#[component]
pub fn Preferences() -> Element {
    let navigator = use_navigator();

    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let mut theme_config: Signal<ThemeConfig> = use_context();
    let toast = use_toast();

    let mut selected_theme = use_signal(|| theme_config.read().name.clone());
    let mut selected_dark = use_signal(|| theme_config.read().is_dark);
    let mut is_loading = use_signal(|| false);

    let is_dark_only = move || {
        let t = selected_theme();
        t == "zwipe" || t == "vantablack"
    };

    let mut save = move || {
        is_loading.set(true);
        let dark_mode = if is_dark_only() { true } else { selected_dark() };
        let request = HttpUpdatePreferences {
            theme: Some(selected_theme()),
            dark_mode: Some(dark_mode),
        };
        spawn(async move {
            session.upkeep(client);
            let Some(session_val) = session() else {
                toast.error(
                    "session expired — please log in again".to_string(),
                    ToastOptions::default().duration(Duration::from_millis(3000)),
                );
                is_loading.set(false);
                return;
            };
            match client().update_preferences(request, &session_val).await {
                Ok(prefs) => {
                    theme_config.set(ThemeConfig::from(&prefs));
                    toast.success(
                        "preferences saved".to_string(),
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
                    h2 { "preferences" }
                }

                div { class: "screen-content centered content-enter",
                    div { class: "container-sm",

                        for theme in ALLOWED_THEMES {
                            button {
                                class: if selected_theme() == *theme { "pref-row selected" } else { "pref-row" },
                                onclick: move |_| {
                                    selected_theme.set(theme.to_string());
                                    if *theme == "zwipe" || *theme == "vantablack" {
                                        selected_dark.set(true);
                                    }
                                },
                                "{theme}"
                            }
                        }

                        div {
                            class: if is_dark_only() { "pref-toggle disabled" } else { "pref-toggle" },
                            span { "dark mode" }
                            button {
                                class: "pref-toggle-btn",
                                disabled: is_dark_only(),
                                onclick: move |_| {
                                    selected_dark.set(!selected_dark());
                                },
                                if selected_dark() { "on" } else { "off" }
                            }
                        }
                    }
                }

                div { class: "util-bar",
                    button {
                        class: "util-btn",
                        onclick: move |_| navigator.go_back(),
                        "back"
                    }
                    button {
                        class: "util-btn",
                        disabled: is_loading(),
                        onclick: move |_| save(),
                        if is_loading() { "saving..." } else { "save" }
                    }
                }
            }
        }
    }
}
