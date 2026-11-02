//! User preferences screen for theme and dark mode selection.

use components::{ThemeConfig, ThemeSheet, use_toast};
use dioxus::prelude::*;
use zwipe_core::http::contracts::user::HttpUpdatePreferences;
use zwipe_ui::preferences::{self, PreferenceChange};

use crate::inbound::components::auth::authed::use_authed;
use crate::inbound::components::telemetry::vocabulary::{ProfileScreen, Screen};
use crate::inbound::components::ui_toast;
use crate::outbound::app_icon;

/// The shared theme sheet, saving a kept pick to the account. Selections
/// live-preview against the whole app; Save persists the pick and toasts once
/// the server answers, and a discarded pick toasts once it has wiped back out.
#[component]
pub fn PreferencesSheet(open: Signal<bool>) -> Element {
    let mut theme_config: Signal<ThemeConfig> = use_context();
    let toast = use_toast();
    let authed = use_authed(Screen::Profile(ProfileScreen::Preferences));

    let save = move |picked: ThemeConfig| {
        let change = PreferenceChange::Theme {
            theme: picked.name,
            dark_mode: picked.is_dark,
        };
        let request = HttpUpdatePreferences::from(change.clone());
        spawn(async move {
            if let Some(prefs) = authed
                .run("update_preferences", |c, s| async move {
                    c.update_preferences(request, &s).await
                })
                .await
            {
                let theme = components::theme_from_preferences(&prefs);
                app_icon::follow(&theme);
                theme_config.set(theme);
                ui_toast::raise(toast, change.saved_toast(&prefs));
            }
        });
    };

    rsx! {
        ThemeSheet {
            open,
            theme: theme_config,
            on_save: save,
            on_unchanged: move |()| ui_toast::raise(toast, preferences::theme_unchanged()),
        }
    }
}
