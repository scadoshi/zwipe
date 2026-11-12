//! The deck page's List | Stacks choice, kept in browser storage so the page
//! opens the way it was last left.

use zwipe_ui::deck_cards::stacks::CardsView;

use crate::outbound::storage::{BrowserStorage, Storage};

/// The storage key the choice lives under.
const KEY: &str = "zwiper-web.deck-cards-view";

/// Saves and loads the deck page's [`CardsView`], in `S`.
#[derive(Debug, Clone)]
pub struct CardsViewStore<S> {
    storage: S,
}

impl CardsViewStore<BrowserStorage> {
    /// The store over the browser's `localStorage`.
    pub fn browser() -> Self {
        Self::new(BrowserStorage)
    }
}

impl<S: Storage> CardsViewStore<S> {
    /// The store over `storage`.
    pub fn new(storage: S) -> Self {
        Self { storage }
    }

    /// The choice last saved, or the list when none was or it is unreadable.
    pub fn load(&self) -> CardsView {
        self.storage
            .read(KEY)
            .and_then(|key| CardsView::from_key(&key))
            .unwrap_or_default()
    }

    /// Stores `view`, logging a failure; a lost write only means the next
    /// visit opens on the list.
    pub fn save(&self, view: CardsView) {
        if let Err(e) = self.storage.write(KEY, view.key()) {
            tracing::debug!("cards view not saved: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::outbound::storage::MemoryStorage;

    #[test]
    fn a_saved_choice_loads_back() {
        let store = CardsViewStore::new(MemoryStorage::default());
        assert_eq!(
            store.load(),
            CardsView::List,
            "never saved opens on the list"
        );
        store.save(CardsView::Stacks);
        assert_eq!(store.load(), CardsView::Stacks);
        store.save(CardsView::List);
        assert_eq!(store.load(), CardsView::List);
    }

    #[test]
    fn the_deck_page_keeps_the_key_it_has_always_had() {
        assert_eq!(KEY, "zwiper-web.deck-cards-view");
    }
}
