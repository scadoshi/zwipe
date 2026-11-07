//! The account's settings fetched and applied: the pips and the swipe
//! confirmations as they are, and the theme by
//! [`zwipe_ui::theme_sync::reconcile`], so the latest choice wins between
//! this device and the server. Home runs it as it loads, and the root
//! each time the app comes back to the foreground, so a theme chosen on
//! another device shows without waiting for Home.

use components::ThemeConfig;
use dioxus::prelude::*;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::user::models::settings::PipStyle;
use zwipe_core::http::contracts::user::HttpUpdateSettings;
use zwipe_ui::theme_sync::{Sync, reconcile};

use crate::inbound::components::auth::ensure_session::EnsureFresh;
use crate::outbound::session::Persist;
use crate::outbound::{app_icon, theme_store};

/// Fetches the settings and applies them, when a session is held. It waits
/// for a usable access token first. A failure logs and changes nothing.
pub fn sync_settings(
    mut session: Signal<Option<Session>>,
    client: Signal<ZwipeClient>,
    mut theme: Signal<ThemeConfig>,
    mut pip_style: Signal<PipStyle>,
) {
    if session.peek().is_none() {
        return;
    }
    spawn(async move {
        let current = match session.ensure_fresh(client).await {
            Ok(current) => current,
            Err(e) => {
                tracing::warn!("settings fetch skipped: {e}");
                return;
            }
        };
        let http = client.peek().clone();
        let settings = match http.fetch_settings(&current).await {
            Ok(settings) => settings,
            Err(e) => {
                tracing::warn!("settings fetch failed: {e}");
                return;
            }
        };
        pip_style.set(settings.pip_style);
        // A swipe confirmations choice made on another device: keep it in the
        // stored session, where the swipe screens read it.
        let held = session.peek().clone();
        if let Some(mut held) = held
            && held.settings.swipe_confirmations != settings.swipe_confirmations
        {
            held.settings.swipe_confirmations = settings.swipe_confirmations;
            held.infallible_save();
            session.set(Some(held));
        }
        let fetched = components::theme_from_settings(&settings);
        let shown = theme.peek().clone();
        match reconcile(&shown, theme_store::load_server_seen().as_ref(), &fetched) {
            Sync::Keep => theme_store::save_server_seen(&fetched),
            Sync::Apply(newer) => {
                theme_store::save_server_seen(&newer);
                app_icon::follow(&newer);
                theme.set(newer);
            }
            Sync::Push(newer) => {
                let request = HttpUpdateSettings {
                    theme: Some(newer.name.clone()),
                    dark_mode: Some(newer.is_dark),
                    exclude_universes_beyond: None,
                    universes_beyond_exceptions: None,
                    pip_style: None,
                    swipe_confirmations: None,
                };
                match http.update_settings(request, &current).await {
                    Ok(saved) => {
                        theme_store::save_server_seen(&components::theme_from_settings(&saved));
                    }
                    Err(e) => tracing::warn!("theme push failed: {e}"),
                }
            }
        }
    });
}
