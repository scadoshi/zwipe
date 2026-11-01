//! The Add screen's search stack parked per deck, so coming back to a deck with the same filter resumes where swiping stopped.

use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;

use crate::stack::AddAction;

/// Most decks kept parked, the least recently used dropped first. The server's per-user deck cap.
pub const MAX_PARKED_DECKS: usize = 20;

/// Swiped cards kept behind the cursor when parking: the undo depth after coming back.
pub const PARKED_BEHIND_CARDS: usize = 50;

/// One deck's parked search stack.
#[derive(Debug, Clone, PartialEq)]
pub struct ParkedStack {
    /// The stack's cards.
    pub cards: Vec<Card>,
    /// The cursor.
    pub index: usize,
    /// The swipes that reach the kept cards.
    pub history: Vec<AddAction>,
    /// The filter that produced the stack; another filter makes the park stale.
    pub filter: CardQueryBuilder,
    /// Whether paging had already run dry.
    pub exhausted: bool,
}

/// Parked stacks, most recently used first.
#[derive(Debug, Default)]
pub struct AddStackCache {
    entries: Vec<(Uuid, ParkedStack)>,
}

impl AddStackCache {
    /// Parks a deck's stack: keeps [`PARKED_BEHIND_CARDS`] swiped cards and the history that reaches them, puts the deck first, and drops the oldest past [`MAX_PARKED_DECKS`].
    pub fn park(&mut self, deck_id: Uuid, mut parked: ParkedStack) {
        let cut = parked.index.saturating_sub(PARKED_BEHIND_CARDS);
        if cut > 0 {
            parked.cards.drain(..cut.min(parked.cards.len()));
            parked.index = parked.index.saturating_sub(cut);
        }
        let excess = parked.history.len().saturating_sub(parked.index);
        parked.history.drain(..excess);

        self.entries.retain(|(id, _)| *id != deck_id);
        self.entries.insert(0, (deck_id, parked));
        self.entries.truncate(MAX_PARKED_DECKS);
    }

    /// Takes a deck's parked stack out of the cache.
    pub fn take(&mut self, deck_id: Uuid) -> Option<ParkedStack> {
        let at = self.entries.iter().position(|(id, _)| *id == deck_id)?;
        Some(self.entries.remove(at).1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::card;

    fn parked(cards: usize, index: usize, history: usize) -> ParkedStack {
        ParkedStack {
            cards: (0..cards).map(|i| card(&i.to_string())).collect(),
            index,
            history: vec![AddAction::Skip; history],
            filter: CardQueryBuilder::default(),
            exhausted: false,
        }
    }

    #[test]
    fn parking_trims_to_fifty_behind_and_the_history_that_reaches_them() {
        let mut cache = AddStackCache::default();
        let deck = Uuid::new_v4();
        cache.park(deck, parked(80, 70, 70));
        let back = cache.take(deck).unwrap();
        assert_eq!(back.index, 50);
        assert_eq!(back.cards.len(), 60);
        assert_eq!(back.cards[0].scryfall_data.name, "20");
        assert_eq!(back.history.len(), 50);
        assert!(cache.take(deck).is_none(), "taking removes it");
    }

    #[test]
    fn a_short_stack_parks_untouched() {
        let mut cache = AddStackCache::default();
        let deck = Uuid::new_v4();
        let stack = parked(10, 3, 3);
        cache.park(deck, stack.clone());
        assert_eq!(cache.take(deck), Some(stack));
    }

    #[test]
    fn the_oldest_deck_drops_past_twenty() {
        let mut cache = AddStackCache::default();
        let decks: Vec<Uuid> = (0..21).map(|_| Uuid::new_v4()).collect();
        for deck in &decks {
            cache.park(*deck, parked(1, 0, 0));
        }
        assert!(cache.take(decks[0]).is_none());
        assert!(cache.take(decks[20]).is_some());
        cache.park(decks[1], parked(1, 0, 0));
        assert!(cache.take(decks[1]).is_some(), "parking again refreshes");
    }

    #[test]
    fn a_cursor_past_the_end_parks_without_panicking() {
        let mut cache = AddStackCache::default();
        let deck = Uuid::new_v4();
        cache.park(deck, parked(10, 100, 3));
        let back = cache.take(deck).unwrap();
        assert!(back.cards.is_empty(), "every card was behind the cursor");
        assert_eq!(back.index, 50);
    }
}
