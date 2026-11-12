//! Each card list's List | Stacks choice, kept in browser storage so the list
//! opens the way it was last left. Every place with the switch keeps its own.

use zwipe_ui::deck_cards::stacks::CardsView;

use crate::outbound::storage::{BrowserStorage, Storage};

/// A card list with the List | Stacks switch, which keeps its own choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CardsViewPlace {
    /// The deck page's cards.
    DeckPage,
    /// Playstyle's Suggested commanders.
    SuggestedCommanders,
    /// Playstyle's Staples.
    Staples,
    /// Activity's Most skipped.
    MostSkipped,
    /// The Commander maybeboard.
    CommanderMaybeboard,
}

impl CardsViewPlace {
    /// The storage key the place's choice lives under.
    #[must_use]
    pub fn key(self) -> &'static str {
        match self {
            Self::DeckPage => "zwiper-web.deck-cards-view",
            Self::SuggestedCommanders => "zwiper-web.view.suggested-commanders",
            Self::Staples => "zwiper-web.view.staples",
            Self::MostSkipped => "zwiper-web.view.most-skipped",
            Self::CommanderMaybeboard => "zwiper-web.view.commander-maybeboard",
        }
    }
}

/// Saves and loads one place's [`CardsView`], in `S`.
#[derive(Debug, Clone)]
pub struct CardsViewStore<S> {
    storage: S,
    place: CardsViewPlace,
}

impl CardsViewStore<BrowserStorage> {
    /// The store for `place` over the browser's `localStorage`.
    pub fn browser(place: CardsViewPlace) -> Self {
        Self::new(BrowserStorage, place)
    }
}

impl<S: Storage> CardsViewStore<S> {
    /// The store for `place` over `storage`.
    pub fn new(storage: S, place: CardsViewPlace) -> Self {
        Self { storage, place }
    }

    /// The choice last saved here, or the list when none was or it is
    /// unreadable.
    pub fn load(&self) -> CardsView {
        self.storage
            .read(self.place.key())
            .and_then(|key| CardsView::from_key(&key))
            .unwrap_or_default()
    }

    /// Stores `view`, logging a failure; a lost write only means the next
    /// visit opens on the list.
    pub fn save(&self, view: CardsView) {
        if let Err(e) = self.storage.write(self.place.key(), view.key()) {
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
        let store = CardsViewStore::new(MemoryStorage::default(), CardsViewPlace::DeckPage);
        store.save(CardsView::Stacks);
        assert_eq!(store.load(), CardsView::Stacks);
        store.save(CardsView::List);
        assert_eq!(store.load(), CardsView::List);
    }

    /// The same storage, read and written for `place`.
    fn moved<S: Storage>(store: CardsViewStore<S>, place: CardsViewPlace) -> CardsViewStore<S> {
        CardsViewStore { place, ..store }
    }

    #[test]
    fn each_place_keeps_its_own_choice_and_starts_on_the_list() {
        let staples = CardsViewStore::new(MemoryStorage::default(), CardsViewPlace::Staples);
        assert_eq!(
            staples.load(),
            CardsView::List,
            "a place never set opens on the list"
        );
        let suggested = moved(staples, CardsViewPlace::SuggestedCommanders);
        suggested.save(CardsView::Stacks);
        assert_eq!(suggested.load(), CardsView::Stacks);
        let staples = moved(suggested, CardsViewPlace::Staples);
        assert_eq!(staples.load(), CardsView::List, "Staples keeps its own");
        let maybeboard = moved(staples, CardsViewPlace::CommanderMaybeboard);
        assert_eq!(maybeboard.load(), CardsView::List);
        let suggested = moved(maybeboard, CardsViewPlace::SuggestedCommanders);
        assert_eq!(
            suggested.load(),
            CardsView::Stacks,
            "the same storage throughout"
        );
    }

    #[test]
    fn the_deck_page_keeps_the_key_it_has_always_had() {
        assert_eq!(CardsViewPlace::DeckPage.key(), "zwiper-web.deck-cards-view");
    }
}
