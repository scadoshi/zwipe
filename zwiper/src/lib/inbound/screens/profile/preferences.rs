//! User preferences screen for theme and dark mode selection.

use crate::inbound::components::{
    auth::authed::use_authed,
    telemetry::vocabulary::{ProfileScreen, Screen},
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use zwipe_components::{TOAST_QUICK, ThemeSheet};
use zwipe_core::{
    domain::user::models::theme::ThemeConfig, http::contracts::user::HttpUpdatePreferences,
};

/// The shared theme sheet, saving a kept pick to the account. Selections
/// live-preview against the whole app; Save persists the pick and toasts once
/// the server answers, and a discarded pick toasts once it has wiped back out.
#[component]
pub fn PreferencesSheet(open: Signal<bool>) -> Element {
    let mut theme_config: Signal<ThemeConfig> = use_context();
    let toast = use_toast();
    let authed = use_authed(Screen::Profile(ProfileScreen::Preferences));

    let save = move |picked: ThemeConfig| {
        let request = HttpUpdatePreferences {
            theme: Some(picked.name),
            dark_mode: Some(picked.is_dark),
            exclude_universes_beyond: None,
            universes_beyond_exceptions: None,
        };
        spawn(async move {
            if let Some(prefs) = authed
                .run("update_preferences", |c, s| async move {
                    c.update_preferences(request, &s).await
                })
                .await
            {
                theme_config.set(ThemeConfig::from(&prefs));
                toast.success(
                    "Theme saved".to_string(),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
            }
        });
    };

    rsx! {
        ThemeSheet {
            open,
            theme: theme_config,
            on_save: save,
            on_unchanged: move |()| {
                toast.info(
                    "Theme unchanged".to_string(),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
            },
        }
    }
}
