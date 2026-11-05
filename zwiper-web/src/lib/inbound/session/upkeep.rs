//! The root's session context and the loop that keeps it alive.

use components::ThemeConfig;
use dioxus::prelude::*;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::user::models::settings::PipColor;
use zwipe_core::http::contracts::metrics::AnonymousEventKind;
use zwipe_ui::session;
use zwipe_ui::timing::UPKEEP_INTERVAL;

use crate::config::APP_VERSION;
use crate::domain::theme;
use crate::inbound::root_signal::use_root_signal;
use crate::inbound::session::ensure_fresh::ensure_fresh;
use crate::inbound::session::funnel;
use crate::outbound::session_store::SessionStore;
use crate::outbound::theme_store::ThemeStore;
use crate::outbound::timer;

/// Whether this build is below the server's minimum version. Provided as
/// context; the root shows the update screen in place of the router while it
/// is true.
#[derive(Debug, Clone, Copy)]
pub struct UpgradeRequired(Signal<bool>);

impl UpgradeRequired {
    /// Reads the gate, subscribing the caller to changes.
    pub fn required(&self) -> bool {
        (self.0)()
    }
}

/// Provides the session, the API client (from the [`ZwipeClient`] the launch
/// put in the root context), the theme and the pip style to
/// everything under the root, and starts the upkeep loop. Call once, in the
/// root component.
///
/// The session and client signals live in the root scope: the refresh writes
/// the session from a task that outlives any screen.
pub fn use_session_upkeep() -> UpgradeRequired {
    let session: Signal<Option<Session>> =
        use_root_signal(|| SessionStore::browser().infallible_load());
    use_context_provider(|| session);

    let built: ZwipeClient = use_context();
    let client = use_root_signal(|| built);
    use_context_provider(|| client);

    let theme: Signal<ThemeConfig> = use_signal(|| {
        let stored = ThemeStore::browser().load();
        theme::initial_theme(session.peek().as_ref().map(|s| &s.settings), stored)
    });
    use_context_provider(|| theme);
    use_effect(move || ThemeStore::browser().save(&theme.read()));

    let pip_style: Signal<PipColor> = use_signal(|| {
        session
            .peek()
            .as_ref()
            .map(|s| s.settings.pip_style)
            .unwrap_or_default()
    });
    use_context_provider(|| pip_style);

    // A logged-out visit opens the funnel; a logged-in one is already a user.
    use_hook(|| {
        if session.peek().is_none() {
            funnel::record(client, AnonymousEventKind::AppOpened);
        }
    });

    let mut upgrade_required = use_signal(|| false);
    use_context_provider(|| UpgradeRequired(upgrade_required));

    use_future(move || async move {
        loop {
            let _ = ensure_fresh(session, client).await;
            let http = client.peek().clone();
            let current = *upgrade_required.peek();
            // A failed check leaves the gate as it was.
            if let Ok(min) = http.fetch_min_client_version().await {
                let required =
                    session::update_required(current, APP_VERSION, Some(&min.min_version));
                if required != current {
                    tracing::info!(min = %min.min_version, required, "min-version gate changed");
                    upgrade_required.set(required);
                }
            }
            // The first round runs at once.
            timer::sleep(UPKEEP_INTERVAL).await;
        }
    });

    UpgradeRequired(upgrade_required)
}
