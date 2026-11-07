//! Saving the theme to the account, shared by Settings' Theme row and the
//! main menu's theme buttons.
//!
//! Both drive a `zwipe_ui::screens::config` model and draw in the
//! app-wide `Signal<ThemeConfig>`. What they share sits here: the sheet that
//! hands a kept pick on as an action, the request a save effect sends, and
//! what a saved theme does to the app (the shell's theme and the app icon
//! that follows it). Signed out, a save answers itself with the pick kept
//! on the device (the session upkeep stores every theme change) and a
//! profile's theme takes over at login.

use components::{ThemeConfig, ThemeSheet};
use dioxus::prelude::*;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::user::HttpUpdateSettings;
use zwipe_ui::pending_settings::PendingSettings;
use zwipe_ui::screens::config::{Action, SaveTicket, Theme};
use zwipe_ui::settings::SettingChange;

use crate::inbound::components::auth::authed::Authed;
use crate::outbound::{app_icon, pending_settings_store, theme_store};

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
/// `dispatch`: [`Action::Saved`] with the server's settings, or
/// [`Action::SaveFailed`] once `authed` has reported the failure and raised
/// its error toast. Without a session the reply is the pick kept on the
/// device. The change stays pending on the device until the server answers,
/// so one the app dies before hearing back about goes up on the next launch.
pub fn save(
    authed: Authed,
    session: Signal<Option<Session>>,
    ticket: SaveTicket,
    change: SettingChange,
    dispatch: impl Fn(Action) + 'static,
) {
    let Some(user_id) = session.peek().as_ref().map(|s| s.user.id) else {
        dispatch(Action::Saved {
            ticket,
            settings: Box::new(change.kept_on_device()),
        });
        return;
    };
    let sent = change.clone();
    pending_settings_store::store(Some(&PendingSettings::record(
        pending_settings_store::load(),
        user_id,
        change.clone(),
    )));
    let request = HttpUpdateSettings::from(change);
    spawn(async move {
        let reply = authed
            .try_run("update_preferences", |c, s| async move {
                c.update_settings(request, &s).await
            })
            .await;
        let left = pending_settings_store::load().and_then(|p| p.answered(&sent));
        pending_settings_store::store(left.as_ref());
        dispatch(match reply {
            Ok(settings) => Action::Saved {
                ticket,
                settings: Box::new(settings),
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
    theme_store::save_server_seen(&theme);
    app_icon::follow(&theme);
    theme_config.set(theme);
}

/// The kit's theme sheet over the app's live theme, handing each pick on as
/// an action. A tap draws the whole app in the pick and hands it on as
/// [`Action::KeepTheme`], which persists it and toasts once the server
/// answers; the sheet stays open to keep browsing.
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
        }
    }
}
