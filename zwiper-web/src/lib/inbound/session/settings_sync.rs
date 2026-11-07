//! The account's settings fetched and applied: the pips as they are, and the
//! theme by `domain::theme::reconcile`, so the latest choice wins between
//! this browser and the server. Home runs it as it loads, and the root each
//! time the tab comes back to the foreground, so a theme chosen on another
//! device shows without waiting for Home.

use components::ThemeConfig;
use dioxus::prelude::*;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::user::models::settings::PipStyle;
use zwipe_core::http::contracts::user::HttpUpdateSettings;

use crate::domain::theme::{self, ThemeSync};
use crate::inbound::session::ensure_fresh::ensure_fresh;
use crate::outbound::theme_store::ThemeStore;

/// Fetches the settings and applies them, when a session is held. A load
/// starts on the stored session, whose access token is a placeholder until
/// the first refresh, so it waits for a usable one first. A failure logs and
/// changes nothing.
pub fn sync_settings(
    session: Signal<Option<Session>>,
    client: Signal<ZwipeClient>,
    mut theme: Signal<ThemeConfig>,
    mut pip_style: Signal<PipStyle>,
) {
    if session.peek().is_none() {
        return;
    }
    spawn(async move {
        let current = match ensure_fresh(session, client).await {
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
        let store = ThemeStore::browser();
        let fetched = components::theme_from_settings(&settings);
        let shown = theme.peek().clone();
        match theme::reconcile(&shown, store.load_server().as_ref(), &fetched) {
            ThemeSync::Keep => store.save_synced(&fetched),
            ThemeSync::Apply(newer) => {
                store.save_synced(&newer);
                theme.set(newer);
            }
            ThemeSync::Push(newer) => {
                let request = HttpUpdateSettings {
                    theme: Some(newer.name.clone()),
                    dark_mode: Some(newer.is_dark),
                    exclude_universes_beyond: None,
                    universes_beyond_exceptions: None,
                    pip_style: None,
                    swipe_confirmations: None,
                };
                match http.update_settings(request, &current).await {
                    Ok(saved) => store.save_synced(&components::theme_from_settings(&saved)),
                    Err(e) => tracing::warn!("theme push failed: {e}"),
                }
            }
        }
    });
}

/// Sends `true` each time the tab comes back to the foreground.
pub const VISIBLE_JS: &str = "document.addEventListener('visibilitychange', () => { if (document.visibilityState === 'visible') dioxus.send(true); });";
