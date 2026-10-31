//! Each card screen's filter per deck, so a filter never leaks from one screen or deck to another.

use std::collections::HashMap;

use uuid::Uuid;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;

/// Which screen a remembered filter belongs to. Add's two sources keep separate filters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum FilterScope {
    /// The deck cards view.
    Cards,
    Remove,
    /// Add, swiping search results.
    AddSearch,
    /// Add, swiping the maybeboard.
    AddMaybeboard,
}

/// Remembered filters by screen and deck.
#[derive(Debug, Default)]
pub(crate) struct FilterStore {
    entries: HashMap<(FilterScope, Uuid), CardQueryBuilder>,
}

impl FilterStore {
    /// The filter last parked for this screen and deck.
    pub(crate) fn restore(&self, scope: FilterScope, deck_id: Uuid) -> Option<CardQueryBuilder> {
        self.entries.get(&(scope, deck_id)).cloned()
    }

    /// Remembers a filter, replacing the one before.
    pub(crate) fn park(&mut self, scope: FilterScope, deck_id: Uuid, filter: CardQueryBuilder) {
        self.entries.insert((scope, deck_id), filter);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_stay_with_their_screen_and_deck() {
        let (deck, other) = (Uuid::new_v4(), Uuid::new_v4());
        let mut store = FilterStore::default();
        store.park(
            FilterScope::Cards,
            deck,
            CardQueryBuilder::with_name_contains("bolt"),
        );
        assert_eq!(
            store
                .restore(FilterScope::Cards, deck)
                .unwrap()
                .name_contains(),
            Some("bolt")
        );
        assert!(store.restore(FilterScope::Remove, deck).is_none());
        assert!(store.restore(FilterScope::Cards, other).is_none());
        store.park(FilterScope::Cards, deck, CardQueryBuilder::default());
        assert!(store.restore(FilterScope::Cards, deck).unwrap().is_empty());
    }
}
