//! Background session upkeep loop and app context providers.
//!
//! Periodically keeps the access token fresh via the single-flight
//! [`EnsureFresh`] helper, and initializes the app-wide Dioxus context
//! (session, client, card search state, theme, telemetry buffer).

use crate::{
    inbound::{
        components::{
            auth::ensure_session::EnsureFresh,
            catalog_cache::use_catalog_cache,
            hint_host::HintTopic,
            telemetry::{
                anonymous::record_anonymous_event,
                flush_loop::{spawn_usage_flusher, spawn_visibility_flusher},
                usage_buffer::UsageBuffer,
            },
        },
        screens::deck::card::components::{
            action_history::AddAction, add_stack_cache::use_add_stack_cache,
            card_stack::use_card_stack, filter_store::use_filter_store,
        },
    },
    outbound::{
        client::{
            ZwipeClient, changelog::get_changelog::ClientGetChangelog,
            version::get_min_client_version::ClientGetMinClientVersion,
        },
        session::Persist,
        theme_store::PersistTheme,
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
    http::contracts::{changelog::HttpChangelog, metrics::AnonymousEventKind},
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

/// App-wide changelog cache, populated by a background fetch at startup.
///
/// Fetched once per launch and held for the session (see [`spawn_upkeeper`]) so
/// opening the Changelog screen is instant after the first launch. The screen
/// reads this: `Loading` shows a skeleton, `Loaded` renders the fetched copy,
/// and `Failed` falls back to the copy compiled into the binary.
#[derive(Clone, PartialEq)]
pub enum ChangelogCache {
    /// Startup fetch still in flight.
    Loading,
    /// Fetched successfully from the server.
    Loaded(HttpChangelog),
    /// Fetch finished but failed; consumers use the compiled-in copy.
    Failed,
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

    // Remembered filters, one per (screen, deck) — each deck-card screen
    // provides its own filter signal seeded from here and parks it back on
    // leave (see filter_store.rs).
    let filter_store = use_filter_store();
    use_context_provider(|| filter_store);

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

    // On-demand hint channel: an InfoButton anywhere posts a HintTopic here and
    // the app-root HintHost renders the dialog (hint_host.rs). One Option = one
    // hint at a time.
    let hint_topic: Signal<Option<HintTopic>> = use_signal(|| None);
    use_context_provider(|| hint_topic);

    // Home flavor card — cached above the router with a TTL (see FlavorCard).
    let flavor_card: Signal<Option<FlavorCard>> = use_signal(|| None);
    use_context_provider(|| flavor_card);

    // Changelog — fetched once in the background at startup and cached above the
    // router for the session, so opening the Changelog screen is instant. The
    // screen shows a skeleton while this is Loading and falls back to the
    // compiled-in copy if it Fails. Public, so it runs even logged out.
    let mut changelog_cache = use_signal(|| ChangelogCache::Loading);
    use_context_provider(|| changelog_cache);
    // use_future (not bare spawn) so the fetch runs exactly once, not on every
    // re-render of this root component (e.g. when the min-version gate flips).
    use_future(move || async move {
        // Bind before awaiting so the peek guard is dropped, not held across it.
        let http = client.peek().clone();
        match http.get_changelog().await {
            Ok(changelog) => changelog_cache.set(ChangelogCache::Loaded(changelog)),
            Err(error) => {
                tracing::debug!(?error, "changelog fetch failed; using compiled-in copy");
                changelog_cache.set(ChangelogCache::Failed);
            }
        }
    });

    // Catalog cache — slow-changing filter metadata (artists, sets, keywords,
    // oracle words, card types, card roles, oracle tags) prefetched in the
    // background at startup and held above the router with a 1-day TTL, so filter
    // sheets / pickers / the dictionary read the cache instead of each firing a
    // fetch on open. Public catalogs send no auth (Cloudflare HITs stay warm);
    // deck tags are authed and warm once a session exists. See catalog_cache.rs.
    let catalog_cache = use_catalog_cache();
    use_context_provider(|| catalog_cache);
    // use_future (not bare spawn) so the prefetch runs exactly once, not on every
    // re-render of this root (e.g. when the min-version gate flips).
    use_future(move || async move {
        catalog_cache.prefetch_public(client);
    });
    // Deck tags need a session; (re)warm when one appears (login / cold-start
    // restore). Single-flight in the cache dedupes repeat runs.
    use_effect(move || {
        if let Some(session) = session() {
            catalog_cache.ensure_deck_tags(client, session);
        }
    });

    // Theme — a live session's preferences win (freshest for this account),
    // else the last-used theme cached locally (so pre-auth screens render in it
    // even logged out / after a device-to-device change), else the default.
    let theme = use_signal(|| {
        session
            .peek()
            .as_ref()
            .map(|s| ThemeConfig::from(&s.preferences))
            .or_else(ThemeConfig::infallible_load)
            .unwrap_or_default()
    });
    use_context_provider(|| theme);

    // Persist every theme change at one point (login, prefs update, picker), so
    // the choice survives logout and themes the next launch's pre-auth screens.
    use_effect(move || theme.read().infallible_save());

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
