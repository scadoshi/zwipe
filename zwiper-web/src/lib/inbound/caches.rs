//! What the root keeps for every screen: data fetched once and shared, so a
//! screen opens on it instead of fetching again.

use dioxus::prelude::*;

use crate::domain::counts::CachedCounts;
use crate::outbound::counts_store::CountsStore;

/// Provides the root caches to everything under the root. Call once, in the
/// root component.
///
/// - `Signal<Option<CachedCounts>>`: everybody's counters for the heroes,
///   starting from the copy an earlier visit stored.
pub fn use_root_caches() {
    let counts: Signal<Option<CachedCounts>> =
        use_signal(|| CountsStore::browser().load().map(CachedCounts::restored));
    use_context_provider(|| counts);
}
