//! The List | Stacks choice of the deck page and of Collection, each kept in browser storage under
//! its own key so each page opens the way it was last left.

use zwipe_ui::deck_cards::stacks::CardsView;
use zwipe_ui::screens::collection_list::stacks::DEFAULT_VIEW as COLLECTION_DEFAULT;

use crate::outbound::storage::{BrowserStorage, Storage};

/// The storage key the deck page's choice lives under.
const KEY: &str = "zwiper-web.deck-cards-view";

/// The storage key Collection's choice lives under.
const COLLECTION_KEY: &str = "zwiper-web.collection-cards-view";

/// Which page's choice a store keeps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Page {
    /// The deck page, which opens on the list until a choice is saved.
    Deck,
    /// Collection, which opens on its stacks until a choice is saved.
    Collection,
}

impl Page {
    /// The storage key.
    const fn key(self) -> &'static str {
        match self {
            Self::Deck => KEY,
            Self::Collection => COLLECTION_KEY,
        }
    }

    /// The view before any choice is saved.
    fn fallback(self) -> CardsView {
        match self {
            Self::Deck => CardsView::default(),
            Self::Collection => COLLECTION_DEFAULT,
        }
    }
}

/// Saves and loads one page's [`CardsView`], in `S`.
#[derive(Debug, Clone)]
pub struct CardsViewStore<S> {
    storage: S,
    page: Page,
}

impl CardsViewStore<BrowserStorage> {
    /// The deck page's store over the browser's `localStorage`.
    pub fn browser() -> Self {
        Self::new(BrowserStorage)
    }

    /// Collection's store over the browser's `localStorage`.
    pub fn collection() -> Self {
        Self::collection_in(BrowserStorage)
    }
}

impl<S: Storage> CardsViewStore<S> {
    /// The deck page's store over `storage`.
    pub fn new(storage: S) -> Self {
        Self {
            storage,
            page: Page::Deck,
        }
    }

    /// Collection's store over `storage`.
    pub fn collection_in(storage: S) -> Self {
        Self {
            storage,
            page: Page::Collection,
        }
    }

    /// The choice last saved, or the page's own default when none was or it is unreadable.
    pub fn load(&self) -> CardsView {
        self.storage
            .read(self.page.key())
            .and_then(|key| CardsView::from_key(&key))
            .unwrap_or_else(|| self.page.fallback())
    }

    /// Stores `view`, logging a failure; a lost write only means the next
    /// visit opens on the page's default.
    pub fn save(&self, view: CardsView) {
        if let Err(e) = self.storage.write(self.page.key(), view.key()) {
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
    fn collection_opens_on_stacks_and_keeps_its_own_choice() {
        let collection = CardsViewStore::collection_in(MemoryStorage::default());
        assert_eq!(
            collection.load(),
            CardsView::Stacks,
            "never saved opens on the stacks"
        );
        collection.save(CardsView::List);
        assert_eq!(collection.load(), CardsView::List);
        assert_eq!(
            collection.storage.read(KEY),
            None,
            "the deck page's choice is its own"
        );
    }

    #[test]
    fn the_deck_page_keeps_the_key_it_has_always_had() {
        assert_eq!(KEY, "zwiper-web.deck-cards-view");
    }
}
