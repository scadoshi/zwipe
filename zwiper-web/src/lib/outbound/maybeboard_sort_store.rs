//! The commander maybeboard's sort, kept in browser storage so the screen
//! opens in the order it was last left.

use zwipe_ui::commander::maybeboard::Sort;

use crate::outbound::storage::{BrowserStorage, Storage};

/// The storage key the sort lives under.
const KEY: &str = "zwiper-web.maybeboard-sort";

/// Saves and loads the maybeboard's sort, in `S`.
#[derive(Debug, Clone, Default)]
pub struct MaybeboardSortStore<S> {
    storage: S,
}

impl MaybeboardSortStore<BrowserStorage> {
    /// The store over the browser's `localStorage`.
    pub fn browser() -> Self {
        Self::new(BrowserStorage)
    }
}

impl<S: Storage> MaybeboardSortStore<S> {
    /// A store over `storage`.
    pub fn new(storage: S) -> Self {
        Self { storage }
    }

    /// The sort last saved, or the default (Newest saved) when none was or
    /// the value is unreadable.
    pub fn load(&self) -> Sort {
        self.storage
            .read(KEY)
            .and_then(|key| Sort::from_key(&key))
            .unwrap_or_default()
    }

    /// Stores `sort`, logging a failure; a lost write only means the next
    /// visit opens on the default.
    pub fn save(&self, sort: Sort) {
        if let Err(e) = self.storage.write(KEY, &sort.key()) {
            tracing::debug!("maybeboard sort not saved: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use zwipe_core::domain::card::search_card::card_filter::sort_direction::SortDirection;
    use zwipe_ui::commander::maybeboard::SortBy;

    use super::*;
    use crate::outbound::storage::MemoryStorage;

    #[test]
    fn a_saved_sort_loads_back_and_starts_on_newest() {
        let store = MaybeboardSortStore::new(MemoryStorage::default());
        assert_eq!(store.load().by, SortBy::Newest);
        let sort = Sort {
            by: SortBy::ManaValue,
            direction: SortDirection::Descending,
        };
        store.save(sort);
        assert_eq!(store.load(), sort);
    }
}
