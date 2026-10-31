//! What the root keeps for every screen: data fetched once and shared, so a
//! screen opens on it instead of fetching again.

use dioxus::prelude::*;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::card::scryfall_data::universe::UbFranchiseView;
use zwipe_core::http::contracts::changelog::HttpChangelog;

use crate::domain::counts::CachedCounts;
use crate::outbound::counts_store::CountsStore;

/// A fetch the root runs once per visit.
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

/// The release history, fetched once at the start of a visit, logged in or
/// not, with no retry. The Changelog screen draws from it.
#[derive(Debug, Clone, Copy)]
pub struct ChangelogCache(pub Signal<Load<HttpChangelog>>);

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

/// Provides the root caches to everything under the root and starts their
/// fetches. Call once, in the root component, after the session upkeep
/// (which provides the client).
///
/// - `Signal<Option<CachedCounts>>`: everybody's counters for the heroes,
///   starting from the copy an earlier visit stored.
/// - [`ChangelogCache`] and [`UbFranchises`].
pub fn use_root_caches() {
    let counts: Signal<Option<CachedCounts>> =
        use_signal(|| CountsStore::browser().load().map(CachedCounts::restored));
    use_context_provider(|| counts);

    let client: Signal<ZwipeClient> = use_context();

    let changelog = ChangelogCache(use_signal(|| Load::Loading));
    use_context_provider(|| changelog);
    use_hook(move || {
        let mut cell = changelog.0;
        spawn(async move {
            let http = client.peek().clone();
            match http.fetch_changelog().await {
                Ok(fetched) => cell.set(Load::Loaded(fetched)),
                Err(e) => {
                    tracing::debug!("changelog not fetched: {e}");
                    cell.set(Load::Failed);
                }
            }
        });
    });

    let franchises = UbFranchises(use_signal(|| Load::Loading));
    use_context_provider(|| franchises);
    use_hook(move || franchises.fetch(client));
}
