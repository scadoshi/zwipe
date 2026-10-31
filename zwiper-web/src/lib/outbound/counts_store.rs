//! Everybody's counters, kept in browser storage so the next visit's hero
//! starts from the last copy instead of rolling until the fetch lands.

use zwipe_core::http::contracts::metrics::HttpPublicMetrics;

use crate::outbound::storage::{BrowserStorage, Storage};

/// The storage key the counters live under.
pub const COUNTS_KEY: &str = "zwiper-web.counts";

/// Saves and loads the last counters in `S`.
#[derive(Debug, Clone, Default)]
pub struct CountsStore<S> {
    storage: S,
}

impl CountsStore<BrowserStorage> {
    /// The store over the browser's `localStorage`.
    pub fn browser() -> Self {
        Self::new(BrowserStorage)
    }
}

impl<S: Storage> CountsStore<S> {
    /// A store over `storage`.
    pub fn new(storage: S) -> Self {
        Self { storage }
    }

    /// The stored counters, or `None` when there are none or they do not
    /// parse.
    pub fn load(&self) -> Option<HttpPublicMetrics> {
        let json = self.storage.read(COUNTS_KEY)?;
        serde_json::from_str(&json).ok()
    }

    /// Stores `metrics`, logging a failure. The copy is decoration, so a lost
    /// write costs only the next visit's first look.
    pub fn save(&self, metrics: &HttpPublicMetrics) {
        let result = serde_json::to_string(metrics)
            .map_err(|e| e.to_string())
            .and_then(|json| {
                self.storage
                    .write(COUNTS_KEY, &json)
                    .map_err(|e| e.to_string())
            });
        if let Err(e) = result {
            tracing::debug!("counters not saved: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::outbound::storage::MemoryStorage;

    #[test]
    fn saved_counters_load_back() {
        let store = CountsStore::new(MemoryStorage::default());
        store.save(&HttpPublicMetrics {
            cards_swiped: 30,
            searches: 20,
            decks_created: 10,
        });
        let loaded = store.load().unwrap();
        assert_eq!(
            (loaded.cards_swiped, loaded.searches, loaded.decks_created),
            (30, 20, 10)
        );
    }

    #[test]
    fn nothing_or_garbage_loads_none() {
        let storage = MemoryStorage::default();
        assert!(CountsStore::new(MemoryStorage::default()).load().is_none());
        storage.write(COUNTS_KEY, "nope").unwrap();
        assert!(CountsStore::new(storage).load().is_none());
    }
}
