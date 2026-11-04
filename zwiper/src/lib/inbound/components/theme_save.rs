//! Saving the theme to the account, shared by Settings' Theme row and the
//! main menu's theme row.
//!
//! Both rows drive a `zwipe_ui::screens::config` model and draw in the
//! app-wide `Signal<ThemeConfig>`. What they share sits here: the sheet that
//! hands a kept pick on as an action, the request a save effect sends, and
//! what a saved theme does to the app (the shell's theme and the app icon
//! that follows it).

use components::{ThemeConfig, ThemeSheet};
use dioxus::prelude::*;
use zwipe_core::http::contracts::user::HttpUpdatePreferences;
use zwipe_ui::preferences::PreferenceChange;
use zwipe_ui::screens::config::{Action, SaveTicket, Theme};

use crate::inbound::components::auth::authed::Authed;
use crate::outbound::app_icon;

/// The app's theme value for a theme the model names.
pub fn theme_config_of(theme: Theme) -> ThemeConfig {
    ThemeConfig {
        name: theme.name,
        is_dark: theme.dark_mode,
    }
}

/// The model's theme value for the theme the app draws in.
pub fn theme_of(config: ThemeConfig) -> Theme {
    Theme {
        name: config.name,
        dark_mode: config.is_dark,
    }
}

/// Sends `change` as its sparse update and hands the model's reply to
/// `dispatch`: [`Action::Saved`] with the server's preferences, or
/// [`Action::SaveFailed`] once `authed` has reported the failure and raised
/// its error toast.
pub fn save(
    authed: Authed,
    ticket: SaveTicket,
    change: PreferenceChange,
    dispatch: impl Fn(Action) + 'static,
) {
    let request = HttpUpdatePreferences::from(change);
    spawn(async move {
        let reply = authed
            .try_run("update_preferences", |c, s| async move {
                c.update_preferences(request, &s).await
            })
            .await;
        dispatch(match reply {
            Ok(preferences) => Action::Saved {
                ticket,
                preferences: Box::new(preferences),
            },
            Err(_) => Action::SaveFailed { ticket },
        });
    });
}

/// Draws the app in `theme`, the one the server holds, resolved the way a
/// theme is read from the account, and moves the app icon along with it where
/// the icon follows the theme.
pub fn saved(mut theme_config: Signal<ThemeConfig>, theme: Theme) {
    let theme = theme_config_of(theme).resolve().unwrap_or_default();
    app_icon::follow(&theme);
    theme_config.set(theme);
}

/// The kit's theme sheet over the app's live theme, handing a kept pick on as
/// an action. Selections live-preview against the whole app; the save check
/// hands the pick on as [`Action::KeepTheme`], which persists it and toasts
/// once the server answers, and a discarded pick hands on
/// [`Action::DiscardTheme`] once it has wiped back out.
#[component]
pub fn ThemeSaveSheet(open: Signal<bool>, on: EventHandler<Action>) -> Element {
    let theme_config: Signal<ThemeConfig> = use_context();

    rsx! {
        ThemeSheet {
            open,
            theme: theme_config,
            on_save: move |picked: ThemeConfig| {
                on.call(Action::KeepTheme(theme_of(picked)));
            },
            on_unchanged: move |()| on.call(Action::DiscardTheme { picked: true }),
        }
    }
}
