//! The deck page's List | Stacks choice, kept in browser storage so the page
//! opens the way it was last left.

use zwipe_ui::deck_cards::stacks::CardsView;

use crate::outbound::storage::{BrowserStorage, Storage};

/// The storage key the choice lives under.
pub const CARDS_VIEW_KEY: &str = "zwiper-web.deck-cards-view";

/// Saves and loads the deck page's [`CardsView`], in `S`.
#[derive(Debug, Clone, Default)]
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
    /// A store over `storage`.
    pub fn new(storage: S) -> Self {
        Self { storage }
    }

    /// The choice last saved, or the list when none was or it is
    /// unreadable.
    pub fn load(&self) -> CardsView {
        self.storage
            .read(CARDS_VIEW_KEY)
            .and_then(|key| CardsView::from_key(&key))
            .unwrap_or_default()
    }

    /// Stores `view`, logging a failure; a lost write only means the next
    /// visit opens on the list.
    pub fn save(&self, view: CardsView) {
        if let Err(e) = self.storage.write(CARDS_VIEW_KEY, view.key()) {
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
        store.save(CardsView::Stacks);
        assert_eq!(store.load(), CardsView::Stacks);
        store.save(CardsView::List);
        assert_eq!(store.load(), CardsView::List);
    }

    #[test]
    fn nothing_or_garbage_opens_the_list() {
        assert_eq!(
            CardsViewStore::new(MemoryStorage::default()).load(),
            CardsView::List
        );
        let storage = MemoryStorage::default();
        storage.write(CARDS_VIEW_KEY, "grid").unwrap();
        assert_eq!(CardsViewStore::new(storage).load(), CardsView::List);
    }
}
