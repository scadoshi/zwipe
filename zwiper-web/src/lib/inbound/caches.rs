//! What the root keeps for every screen: data fetched once and shared, so a
//! screen opens on it instead of fetching again.

use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::universe::UbFranchiseView;
use zwipe_core::http::contracts::metrics::HttpLifetimeCounters;
use zwipe_core::http::contracts::suggestions::HttpPlayStyleSummary;
use zwipe_ui::changelog::ChangelogCache;
use zwipe_ui::screens::home::DeckTile;
pub use zwipe_ui::screens::home::FlavorCard;
use zwipe_ui::timing::COUNTS_FRESH;

use crate::domain::counts::CachedCounts;
use crate::inbound::root_signal::use_root_signal;
use crate::outbound::counts_store::CountsStore;

/// A fetch the root runs once per visit and again after a failure.
#[derive(Debug, Clone, PartialEq)]
pub enum Load<T> {
    /// Still out.
    Loading,
    /// Answered.
    Loaded(T),
    /// Answered with an error, logged when it came.
    Failed,
}

impl<T> Load<T> {
    /// The value once loaded.
    pub fn loaded(&self) -> Option<&T> {
        match self {
            Load::Loaded(value) => Some(value),
            Load::Loading | Load::Failed => None,
        }
    }
}

/// The Universes Beyond franchises the Exceptions sheet offers, fetched at
/// the start of a visit and again by [`UbFranchises::ensure`] after a
/// failure.
#[derive(Debug, Clone, Copy)]
pub struct UbFranchises(pub Signal<Load<Vec<UbFranchiseView>>>);

impl UbFranchises {
    /// Fetches the list again when the last fetch failed. Call it from the
    /// screen that needs the list.
    pub fn ensure(self, client: Signal<ZwipeClient>) {
        if matches!(*self.0.peek(), Load::Failed) {
            self.fetch(client);
        }
    }

    fn fetch(self, client: Signal<ZwipeClient>) {
        let mut cell = self.0;
        cell.set(Load::Loading);
        dioxus::core::spawn_forever(async move {
            let http = client.peek().clone();
            match http.fetch_ub_franchises().await {
                Ok(franchises) => cell.set(Load::Loaded(franchises)),
                Err(e) => {
                    tracing::warn!("Universes Beyond franchises not fetched: {e}");
                    cell.set(Load::Failed);
                }
            }
        });
    }
}

/// The commander a `Create deck` button or a right swipe hands to Create
/// deck: Create takes it once, sets the format to Commander and fills the
/// commander with it. Playstyle and the commander maybeboard set it before
/// pushing the route.
#[derive(Debug, Clone, Copy)]
pub struct CreateDeckSeed(pub Signal<Option<Card>>);

/// The user's own counters as last fetched, shared by Home's `Only me` and
/// Activity's header. Never stored between visits.
#[derive(Debug, Clone)]
pub struct CachedMyCounts {
    /// The user they belong to; another user's copy is never shown.
    pub user_id: Uuid,
    /// What the server answered.
    pub metrics: HttpLifetimeCounters,
    /// When it answered.
    pub fetched_at: DateTime<Utc>,
}

impl CachedMyCounts {
    /// Whether this copy serves `user_id` at `now` without a fetch: theirs,
    /// and younger than zwipe-ui's [`COUNTS_FRESH`].
    pub fn is_fresh_for(&self, user_id: Uuid, now: DateTime<Utc>) -> bool {
        let fresh = chrono::Duration::from_std(COUNTS_FRESH).unwrap_or(chrono::Duration::MAX);
        self.user_id == user_id && now.signed_duration_since(self.fetched_at) < fresh
    }
}

/// The playstyle summary Home's Colors panel last drew, so a return to Home
/// shows it at once while the refetch is out.
#[derive(Debug, Clone, PartialEq)]
pub struct CachedPlayStyle {
    /// The user it belongs to; another user's copy is never shown.
    pub user_id: Uuid,
    /// What the server answered.
    pub summary: HttpPlayStyleSummary,
}

/// The last deck list Home fetched, so a return to Home lists the decks
/// at once while the refetch is out.
#[derive(Debug, Clone, PartialEq)]
pub struct CachedDecks {
    /// The user the decks belong to; another user's copy is never shown.
    pub user_id: Uuid,
    /// What the server answered, most recently updated first.
    pub decks: Vec<DeckTile>,
}

/// Provides the root caches to everything under the root and starts their
/// fetches. Call once, in the root component, after the session upkeep
/// (which provides the client).
///
/// - `Signal<Option<CachedCounts>>`: everybody's counters for the heroes,
///   starting from the copy an earlier visit stored.
/// - `Signal<ChangelogCache>`, fetched once at the start of a visit, logged
///   in or not, with no retry; and [`UbFranchises`] in the root scope its
///   fetches run in.
/// - [`CreateDeckSeed`], empty.
/// - `Signal<Option<CachedMyCounts>>`, `Signal<Option<CachedPlayStyle>>`,
///   `Signal<Option<CachedDecks>>` and `Signal<Option<FlavorCard>>` (the
///   featured flavor card, held until the top of the next hour), empty.
pub fn use_root_caches() {
    let counts: Signal<Option<CachedCounts>> =
        use_signal(|| CountsStore::browser().load().map(CachedCounts::restored));
    use_context_provider(|| counts);

    let client: Signal<ZwipeClient> = use_context();

    let changelog: Signal<ChangelogCache> = use_signal(ChangelogCache::default);
    use_context_provider(|| changelog);
    use_hook(move || {
        let mut cell = changelog;
        spawn(async move {
            let http = client.peek().clone();
            match http.fetch_changelog().await {
                Ok(fetched) => cell.set(ChangelogCache::Loaded(fetched)),
                Err(e) => {
                    tracing::debug!("changelog not fetched: {e}");
                    cell.set(ChangelogCache::Failed);
                }
            }
        });
    });

    let seed = CreateDeckSeed(use_signal(|| None));
    use_context_provider(|| seed);
    let my_counts: Signal<Option<CachedMyCounts>> = use_signal(|| None);
    use_context_provider(|| my_counts);
    let play_style: Signal<Option<CachedPlayStyle>> = use_signal(|| None);
    use_context_provider(|| play_style);
    let decks: Signal<Option<CachedDecks>> = use_signal(|| None);
    use_context_provider(|| decks);
    let flavor: Signal<Option<FlavorCard>> = use_signal(|| None);
    use_context_provider(|| flavor);

    let franchises = UbFranchises(use_root_signal(|| Load::Loading));
    use_context_provider(|| franchises);
    use_hook(move || franchises.fetch(client));
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;

    fn mine(user_id: Uuid, at: DateTime<Utc>) -> CachedMyCounts {
        CachedMyCounts {
            user_id,
            metrics: HttpLifetimeCounters {
                user_id,
                swipes_right: 0,
                swipes_left: 0,
                swipes_up: 0,
                swipes_down: 0,
                searches: 0,
                decks_created: 0,
                decks_completed: 0,
                updated_at: at,
            },
            fetched_at: at,
        }
    }

    #[test]
    fn my_counts_serve_their_user_for_a_minute() {
        let user = Uuid::new_v4();
        let at = Utc::now();
        let cached = mine(user, at);
        assert!(cached.is_fresh_for(user, at + Duration::seconds(59)));
        assert!(!cached.is_fresh_for(user, at + Duration::seconds(60)));
        assert!(
            !cached.is_fresh_for(Uuid::new_v4(), at),
            "another user's copy"
        );
    }
}
