//! App-wide catalog cache — slow-changing card/deck filter metadata, fetched
//! once at startup and held above the router for the session.
//!
//! The filter subsections, pickers, and (soon) the oracle-tag dictionary all read
//! the same lists: artists, sets, keywords, oracle words, card types, card roles,
//! oracle tags, and deck tags. Those lists only change when Scryfall / `zervice`
//! refreshes (daily) or when we deploy catalog consts — not when a user opens a
//! filter sheet. Re-fetching on every open is wasteful (mobile latency, redundant
//! origin / Cloudflare work, janky skeletons mid-flow), so we prefetch them in the
//! background at launch and let consumers read the cache instead of each firing
//! their own `use_resource`.
//!
//! TTL is one day, matching the nightly sync cadence and Cloudflare's Rule 1 (24h
//! edge cache on `/api/card/*`). A long-lived process (mobile apps can stay alive
//! for days) revalidates on expiry with stale-while-revalidate, so filters never
//! blank mid-session. Cold start (process killed) clears this and re-prefetches.
//!
//! The eight public card catalogs send **no** `Authorization` header, so they warm
//! even logged out and keep Cloudflare cache HITs working — do not add bearer auth
//! to them. Deck tags are the one authed catalog; they warm once a session exists.

use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use zwipe::inbound::http::ApiError;
use zwipe_core::domain::{
    auth::models::session::Session,
    card::{card_role::CardRoleView, oracle_tag::OracleTag},
    deck::DeckTagView,
};

use crate::outbound::client::{
    ZwipeClient,
    card::{
        get_artists::ClientGetArtists, get_card_roles::ClientGetCardRoles,
        get_card_types::ClientGetCardTypes, get_keywords::ClientGetKeywords,
        get_oracle_tags::ClientGetOracleTags, get_oracle_words::ClientGetOracleWords,
        get_sets::ClientGetSets,
    },
    deck::get_deck_tags::ClientGetDeckTags,
};

/// How long a fetched catalog stays fresh before a read schedules a revalidation.
/// One day, aligned with the nightly sync and Cloudflare's 24h edge cache.
const CATALOG_TTL_HOURS: i64 = 24;

/// State of one cached catalog list.
///
/// Mirrors [`ChangelogCache`](super::auth::session_upkeep::ChangelogCache): a
/// consumer shows a skeleton on `Loading`, renders on `Loaded`, and falls back to
/// an empty list (or its own error state) on `Failed`.
#[derive(Clone, PartialEq)]
pub enum CatalogCell<T> {
    /// No successful fetch yet; a fetch is in flight or about to be.
    Loading,
    /// Fetched successfully, with the instant it was fetched (for TTL).
    Loaded {
        /// The cached list.
        data: T,
        /// When this entry was fetched; drives staleness.
        fetched_at: DateTime<Utc>,
    },
    /// A fetch finished but failed and there is no prior good data to show.
    Failed,
}

impl<T> CatalogCell<T> {
    /// The loaded list, if present (stale or fresh). `None` while Loading/Failed.
    pub fn loaded(&self) -> Option<&T> {
        match self {
            CatalogCell::Loaded { data, .. } => Some(data),
            _ => None,
        }
    }

    /// Whether this is `Loaded` but past its TTL and due for a background refresh.
    fn is_stale(&self) -> bool {
        match self {
            CatalogCell::Loaded { fetched_at, .. } => {
                Utc::now() >= *fetched_at + chrono::Duration::hours(CATALOG_TTL_HOURS)
            }
            _ => false,
        }
    }
}

/// One cached catalog: the cell plus a single-flight guard so N racing consumers
/// (and the startup prefetch) trigger at most one fetch.
pub struct CatalogSlot<T: 'static> {
    cell: Signal<CatalogCell<T>>,
    fetching: Signal<bool>,
}

// Signals are `Copy` for any `T: 'static`, so a slot is too — but a derive would
// wrongly demand `T: Copy`. Hand-write the impls (and thus `CatalogCache`'s).
impl<T: 'static> Clone for CatalogSlot<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: 'static> Copy for CatalogSlot<T> {}

impl<T: Clone + PartialEq + 'static> CatalogSlot<T> {
    /// Reactive handle to the cell — read it with `.read().loaded()`.
    pub fn cell(&self) -> Signal<CatalogCell<T>> {
        self.cell
    }

    /// Kick off a fetch if none is in flight and the data is missing or stale.
    ///
    /// Single-flight: a live fetch short-circuits repeat callers. Fresh `Loaded`
    /// data short-circuits too. Stale-while-revalidate: a stale entry stays
    /// visible while the refetch runs, and a failed *refresh* keeps the last good
    /// data rather than dropping to `Failed`.
    fn refresh<F, Fut>(self, fetch: F)
    where
        F: FnOnce() -> Fut + 'static,
        Fut: std::future::Future<Output = Result<T, ApiError>> + 'static,
    {
        let mut fetching = self.fetching;
        let mut cell = self.cell;

        if *fetching.peek() {
            return;
        }
        let needs_fetch = {
            let current = cell.peek();
            match &*current {
                CatalogCell::Loaded { .. } => current.is_stale(),
                _ => true,
            }
        };
        if !needs_fetch {
            return;
        }

        fetching.set(true);
        spawn(async move {
            match fetch().await {
                Ok(data) => cell.set(CatalogCell::Loaded {
                    data,
                    fetched_at: Utc::now(),
                }),
                Err(error) => {
                    tracing::debug!(?error, "catalog fetch failed");
                    // Don't blow away good (stale) data on a refresh failure.
                    if !matches!(&*cell.peek(), CatalogCell::Loaded { .. }) {
                        cell.set(CatalogCell::Failed);
                    }
                }
            }
            fetching.set(false);
        });
    }
}

/// App-scoped cache of the slow-changing filter catalogs, provided as context in
/// `spawn_upkeeper` and read by filter screens, pickers, and the dictionary.
///
/// A `Copy` handle over per-catalog signals: cheap to pass around, one context
/// lookup, one place to reason about TTL. Public catalogs warm at startup; deck
/// tags warm once a session exists.
#[derive(Clone, Copy)]
pub struct CatalogCache {
    /// Unique artist names (`GET /api/card/artists`).
    pub artists: CatalogSlot<Vec<String>>,
    /// Card set names (`GET /api/card/sets`).
    pub sets: CatalogSlot<Vec<String>>,
    /// Keyword abilities (`GET /api/card/keywords`).
    pub keywords: CatalogSlot<Vec<String>>,
    /// Normalized oracle-text words (`GET /api/card/oracle-words`).
    pub oracle_words: CatalogSlot<Vec<String>>,
    /// Card types (`GET /api/card/types`).
    pub card_types: CatalogSlot<Vec<String>>,
    /// Card-role catalog (`GET /api/card/roles`).
    pub card_roles: CatalogSlot<Vec<CardRoleView>>,
    /// Oracle-tag catalog (`GET /api/card/oracle-tags`) — picker, filter, dictionary.
    pub oracle_tags: CatalogSlot<Vec<OracleTag>>,
    /// Deck-tag catalog (`GET /api/deck/tags`) — authed; warmed after session.
    pub deck_tags: CatalogSlot<Vec<DeckTagView>>,
}

/// Creates the cache (call once, in `spawn_upkeeper`, then provide as context).
///
/// One `use_signal` per field keeps hook order stable across renders.
pub fn use_catalog_cache() -> CatalogCache {
    CatalogCache {
        artists: use_catalog_slot(),
        sets: use_catalog_slot(),
        keywords: use_catalog_slot(),
        oracle_words: use_catalog_slot(),
        card_types: use_catalog_slot(),
        card_roles: use_catalog_slot(),
        oracle_tags: use_catalog_slot(),
        deck_tags: use_catalog_slot(),
    }
}

/// Fresh slot: `Loading`, nothing in flight.
fn use_catalog_slot<T: 'static>() -> CatalogSlot<T> {
    CatalogSlot {
        cell: use_signal(|| CatalogCell::Loading),
        fetching: use_signal(|| false),
    }
}

impl CatalogCache {
    /// Prefetch every public card catalog. Called once at startup; each fetch is
    /// independent single-flight, so a later consumer read is a no-op HIT. Never
    /// sends auth — keeps Cloudflare cache HITs alive.
    pub fn prefetch_public(self, client: Signal<ZwipeClient>) {
        self.ensure_artists(client);
        self.ensure_sets(client);
        self.ensure_keywords(client);
        self.ensure_oracle_words(client);
        self.ensure_card_types(client);
        self.ensure_card_roles(client);
        self.ensure_oracle_tags(client);
    }

    /// Ensure the artist catalog is warm/fresh (public).
    pub fn ensure_artists(self, client: Signal<ZwipeClient>) {
        self.artists.refresh(move || async move {
            let http = client.peek().clone();
            http.get_artists().await
        });
    }

    /// Ensure the set catalog is warm/fresh (public).
    pub fn ensure_sets(self, client: Signal<ZwipeClient>) {
        self.sets.refresh(move || async move {
            let http = client.peek().clone();
            http.get_sets().await
        });
    }

    /// Ensure the keyword catalog is warm/fresh (public).
    pub fn ensure_keywords(self, client: Signal<ZwipeClient>) {
        self.keywords.refresh(move || async move {
            let http = client.peek().clone();
            http.get_keywords().await
        });
    }

    /// Ensure the oracle-words catalog is warm/fresh (public).
    pub fn ensure_oracle_words(self, client: Signal<ZwipeClient>) {
        self.oracle_words.refresh(move || async move {
            let http = client.peek().clone();
            http.get_oracle_words().await
        });
    }

    /// Ensure the card-types catalog is warm/fresh (public).
    pub fn ensure_card_types(self, client: Signal<ZwipeClient>) {
        self.card_types.refresh(move || async move {
            let http = client.peek().clone();
            http.get_card_types().await
        });
    }

    /// Ensure the card-role catalog is warm/fresh (public).
    pub fn ensure_card_roles(self, client: Signal<ZwipeClient>) {
        self.card_roles.refresh(move || async move {
            let http = client.peek().clone();
            http.get_card_roles().await
        });
    }

    /// Ensure the oracle-tag catalog is warm/fresh (public). Consumed by the otag
    /// filter, the deck-tag-seed picker, and the dictionary.
    pub fn ensure_oracle_tags(self, client: Signal<ZwipeClient>) {
        self.oracle_tags.refresh(move || async move {
            let http = client.peek().clone();
            http.get_oracle_tags().await
        });
    }

    /// Ensure the deck-tag catalog is warm/fresh (authed — needs a session).
    pub fn ensure_deck_tags(self, client: Signal<ZwipeClient>, session: Session) {
        self.deck_tags.refresh(move || async move {
            let http = client.peek().clone();
            http.get_deck_tags(&session).await
        });
    }
}
