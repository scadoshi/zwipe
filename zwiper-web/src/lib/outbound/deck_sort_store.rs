//! The deck list's sort, kept in browser storage so Decks opens in the order
//! it was last left.

use zwipe_ui::deck::list::Sort;

use crate::outbound::storage::{BrowserStorage, Storage};

/// The storage key the sort lives under.
pub const DECK_SORT_KEY: &str = "zwiper-web.deck-sort";

/// Saves and loads the deck list's sort, in `S`.
#[derive(Debug, Clone, Default)]
pub struct DeckSortStore<S> {
    storage: S,
}

impl DeckSortStore<BrowserStorage> {
    /// The store over the browser's `localStorage`.
    pub fn browser() -> Self {
        Self::new(BrowserStorage)
    }
}

impl<S: Storage> DeckSortStore<S> {
    /// A store over `storage`.
    pub fn new(storage: S) -> Self {
        Self { storage }
    }

    /// The sort last saved, or the default (Name, ascending) when none was
    /// or the value is unreadable.
    pub fn load(&self) -> Sort {
        self.storage
            .read(DECK_SORT_KEY)
            .and_then(|key| Sort::from_key(&key))
            .unwrap_or_default()
    }

    /// Stores `sort`, logging a failure; a lost write only means the next
    /// visit opens on the default.
    pub fn save(&self, sort: Sort) {
        if let Err(e) = self.storage.write(DECK_SORT_KEY, &sort.key()) {
            tracing::debug!("deck sort not saved: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use zwipe_core::domain::card::search_card::card_filter::sort_direction::SortDirection;
    use zwipe_ui::deck::list::SortBy;

    use super::*;
    use crate::outbound::storage::MemoryStorage;

    #[test]
    fn a_saved_sort_loads_back() {
        let store = DeckSortStore::new(MemoryStorage::default());
        let sort = Sort {
            by: SortBy::Size,
            direction: SortDirection::Descending,
        };
        store.save(sort);
        assert_eq!(store.load(), sort);
    }

    #[test]
    fn nothing_or_garbage_loads_the_default() {
        assert_eq!(
            DeckSortStore::new(MemoryStorage::default()).load(),
            Sort::default()
        );
        let storage = MemoryStorage::default();
        storage.write(DECK_SORT_KEY, "price-sideways").unwrap();
        assert_eq!(DeckSortStore::new(storage).load(), Sort::default());
    }
}
