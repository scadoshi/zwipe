//! Background session upkeep loop and app context providers.
//!
//! Periodically keeps the access token fresh via the single-flight
//! [`EnsureFresh`] helper, and initializes the app-wide Dioxus context
//! (session, client, card search state, theme, telemetry buffer).

use crate::{
    inbound::{
        components::{
            auth::ensure_session::EnsureFresh,
            telemetry::{
                anonymous::record_anonymous_event,
                flush_loop::{spawn_usage_flusher, spawn_visibility_flusher},
                usage_buffer::UsageBuffer,
            },
        },
        screens::deck::card::components::{
            action_history::AddAction, add_stack_cache::use_add_stack_cache,
            card_stack::use_card_stack,
        },
    },
    outbound::{
        client::{ZwipeClient, version::get_min_client_version::ClientGetMinClientVersion},
        session::Persist,
    },
};
use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use std::time::Duration;
use tokio::time::interval;
use zwipe_core::{
    domain::{
        auth::models::session::Session,
        card::{Card, search_card::card_filter::builder::CardQueryBuilder},
        user::models::theme::ThemeConfig,
    },
    http::contracts::metrics::AnonymousEventKind,
    version::version_at_least,
};

/// The running app version, baked in at compile time (matches
/// CFBundleShortVersionString since 1.0.3).
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// How long the Home screen's cached flavor card stays fresh.
const FLAVOR_TTL_HOURS: i64 = 1;

/// A flavor card cached for the Home screen, with the time it goes stale.
///
/// Stored above the router (as `Signal<Option<FlavorCard>>`) so it survives
/// navigation: Home reads it and only refetches when empty or expired, so
/// rapid back-and-forth no longer hammers the rate-limited search endpoint or
/// blanks the quote.
#[derive(Clone)]
pub struct FlavorCard {
    /// The card whose flavor text is shown.
    pub card: Card,
    /// When this entry goes stale and should be refetched.
    pub expires_at: DateTime<Utc>,
}

impl FlavorCard {
    /// Wrap a freshly fetched card with an expiry `FLAVOR_TTL_HOURS` from now.
    pub fn new(card: Card) -> Self {
        Self {
            card,
            expires_at: Utc::now() + chrono::Duration::hours(FLAVOR_TTL_HOURS),
        }
    }

    /// Whether the cached card has passed its TTL and should be refetched.
    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at
    }
}

/// Min-version gate state — true when this build is below the server minimum.
///
/// Newtype so `try_use_context` / `use_context` lookups can't collide with
/// other `Signal<bool>` contexts: a bare `Signal<bool>` here was once grabbed
/// by the filter sheet's collapse lookup, which flashed the blocking
/// "Update required" screen every time Apply was hit on add/remove cards.
#[derive(Clone, Copy)]
pub struct UpgradeRequired(Signal<bool>);

impl UpgradeRequired {
    /// Reactive read — subscribes the caller to gate changes.
    pub fn required(&self) -> bool {
        (self.0)()
    }
}

/// Spawns a background task that periodically refreshes the user session and
/// polls the server's minimum supported app version.
///
/// Also initializes context providers for session, client, card filter, and
/// cards. Returns the [`UpgradeRequired`] gate — true when this build is
/// below the server minimum; the root component swaps the router for a
/// blocking update screen.
pub fn spawn_upkeeper() -> UpgradeRequired {
    tracing::debug!("upkeeper spawned");
    let session = use_signal(Session::infallible_load);
    use_context_provider(|| session);

    let client = use_signal(ZwipeClient::new);
    use_context_provider(|| client);

    // card search state - used by deck card screens
    let filter_builder = use_signal(CardQueryBuilder::default);
    use_context_provider(|| filter_builder);

    // The add screen's search stack (cards, cursor, undo history, animation)
    // — app-scoped so leaving and re-entering the screen resumes mid-stack
    // instead of re-serving already-swiped (and durably skipped) cards.
    let add_stack = use_card_stack::<AddAction>();
    use_context_provider(|| add_stack);

    // Parked add stacks, one per deck (MRU-capped) — leaving the add screen
    // parks the live stack here; returning to that deck restores it.
    let add_stack_cache = use_add_stack_cache();
    use_context_provider(|| add_stack_cache);

    let last_search_filter: Signal<Option<CardQueryBuilder>> = use_signal(|| None);
    use_context_provider(|| last_search_filter);

    // Home flavor card — cached above the router with a TTL (see FlavorCard).
    let flavor_card: Signal<Option<FlavorCard>> = use_signal(|| None);
    use_context_provider(|| flavor_card);

    // Theme — initialize from session preferences if logged in
    let theme = use_signal(|| {
        session
            .peek()
            .as_ref()
            .map(|s| ThemeConfig::from(&s.preferences))
            .unwrap_or_default()
    });
    use_context_provider(|| theme);

    // Usage telemetry buffer (swipe / search counters + suggestion signals).
    // Flushed by two tasks sharing this one buffer: a 30s timer, and a
    // visibility flush that fires the instant the app backgrounds (so a
    // swipe-to-close doesn't lose the last unflushed window).
    // use_hook: spawn exactly once — a plain call here would leak a new flush
    // loop every time the root component re-renders.
    let usage_buffer = use_signal(UsageBuffer::new);
    use_context_provider(|| usage_buffer);
    use_hook(|| {
        spawn_usage_flusher(usage_buffer.peek().clone(), client, session);
        spawn_visibility_flusher(usage_buffer.peek().clone(), client, session);

        // Pre-auth funnel: a logged-out launch is the top of the register
        // funnel. Logged-in launches are already covered by last_active_at.
        if session.peek().is_none() {
            record_anonymous_event(client, AnonymousEventKind::AppOpened);
        }
    });

    // Min-version gate — flipped true when the server says this build is too
    // old. Provided as context (newtyped) so any screen can read it if needed.
    let mut upgrade_required = use_signal(|| false);
    use_context_provider(|| UpgradeRequired(upgrade_required));

    // use_future (not bare spawn) for the same once-only reason as above —
    // the gate flipping re-renders the root, which re-runs this function.
    use_future(move || async move {
        // first tick fires immediately — this is the cold-start refresh
        let mut interval = interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            // single-flight: free of races with any in-flight user action
            let _ = session.ensure_fresh(client).await;

            // Min-version gate check. Fails open: only a successful response
            // can flip the gate — a network hiccup never locks anyone out.
            let http = client.peek().clone();
            if let Ok(min) = http.get_min_client_version().await {
                let required = !version_at_least(APP_VERSION, &min.min_version);
                if required != *upgrade_required.peek() {
                    tracing::info!(
                        app = APP_VERSION,
                        min = %min.min_version,
                        required,
                        "min-version gate changed"
                    );
                    upgrade_required.set(required);
                }
            }
        }
    });

    UpgradeRequired(upgrade_required)
}
