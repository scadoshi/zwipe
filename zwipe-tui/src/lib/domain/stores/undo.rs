//! The per-deck undo log every card screen records into and the deck cards view's Undo walks back, newest first.
//!
//! The Add and Remove screens keep their own gesture undo, and reconcile through this log: before reversing a swipe they take its entry back with [`UndoStore::take_newest`]. `Some` means the change still stands, so they send the reversing call. `None` with the world already reversed means the deck cards Undo did it first, so they skip the call. `None` with the change still standing means the entry aged out of the cap, so they reverse anyway.

use std::collections::HashMap;

use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::deck::{Board, DeckEntry};

/// Entries kept per deck; the oldest drop first.
pub(crate) const MAX_UNDO_ACTIONS: usize = 100;

/// A command zone slot whose printing can change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CommandZoneSlot {
    Commander,
    Partner,
    Background,
    SignatureSpell,
}

/// One change to a deck, with what its undo needs.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum UndoAction {
    /// A card was added; undo deletes it.
    Added { card_id: Uuid, card_name: String },
    /// A card was removed; undo puts it back on its board at `baseline` copies.
    Removed {
        entry: Box<DeckEntry>,
        baseline: i32,
    },
    /// A burst of quantity taps landed; undo applies `-net`.
    QuantityChanged {
        card_id: Uuid,
        card_name: String,
        net: i32,
        baseline: i32,
    },
    /// A card moved boards; undo moves it back to `from`.
    MovedBoard {
        card_id: Uuid,
        card_name: String,
        from: Board,
    },
    /// A deck card changed printing; undo restores `old_card`.
    PrintingChanged { old_card: Box<Card>, new_id: Uuid },
    /// A command zone card changed printing; undo writes `old_card` back to the deck's slot.
    CommandZonePrintingChanged {
        slot: CommandZoneSlot,
        old_card: Box<Card>,
        new_id: Uuid,
    },
}

/// The `board` of a create request that puts a card back on `board`: `None` for the main deck, which is the server's default.
pub(crate) fn create_board_field(board: Board) -> Option<String> {
    match board {
        Board::Deck => None,
        other => Some(other.display_name().to_owned()),
    }
}

/// Appends `action`, dropping the oldest past [`MAX_UNDO_ACTIONS`].
fn push_capped(log: &mut Vec<UndoAction>, action: UndoAction) {
    log.push(action);
    let overflow = log.len().saturating_sub(MAX_UNDO_ACTIONS);
    log.drain(..overflow);
}

/// Removes and returns the newest entry `matches` picks.
fn take_newest_in(
    log: &mut Vec<UndoAction>,
    matches: impl Fn(&UndoAction) -> bool,
) -> Option<UndoAction> {
    let at = log.iter().rposition(matches)?;
    Some(log.remove(at))
}

/// Every deck's undo log.
#[derive(Debug, Default)]
pub(crate) struct UndoStore {
    entries: HashMap<Uuid, Vec<UndoAction>>,
}

impl UndoStore {
    /// The deck's log, oldest first.
    pub(crate) fn log(&self, deck_id: Uuid) -> &[UndoAction] {
        self.entries.get(&deck_id).map_or(&[], Vec::as_slice)
    }

    pub(crate) fn is_empty(&self, deck_id: Uuid) -> bool {
        self.log(deck_id).is_empty()
    }

    pub(crate) fn push(&mut self, deck_id: Uuid, action: UndoAction) {
        push_capped(self.entries.entry(deck_id).or_default(), action);
    }

    /// Takes the deck's newest entry off its log.
    pub(crate) fn pop(&mut self, deck_id: Uuid) -> Option<UndoAction> {
        self.entries.get_mut(&deck_id)?.pop()
    }

    /// Takes back the deck's newest entry `matches` picks, the gesture undo's reconciliation.
    pub(crate) fn take_newest(
        &mut self,
        deck_id: Uuid,
        matches: impl Fn(&UndoAction) -> bool,
    ) -> Option<UndoAction> {
        take_newest_in(self.entries.get_mut(&deck_id)?, matches)
    }

    /// Drops the deck's whole log, as an import does.
    pub(crate) fn clear(&mut self, deck_id: Uuid) {
        self.entries.remove(&deck_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn added(card_id: Uuid, name: &str) -> UndoAction {
        UndoAction::Added {
            card_id,
            card_name: name.to_owned(),
        }
    }

    fn name_of(action: &UndoAction) -> &str {
        match action {
            UndoAction::Added { card_name, .. } => card_name,
            other => panic!("only adds here: {other:?}"),
        }
    }

    #[test]
    fn the_log_drops_its_oldest_past_the_cap() {
        let deck = Uuid::new_v4();
        let mut store = UndoStore::default();
        for i in 0..MAX_UNDO_ACTIONS + 5 {
            store.push(deck, added(Uuid::new_v4(), &i.to_string()));
        }
        let log = store.log(deck);
        assert_eq!(log.len(), MAX_UNDO_ACTIONS);
        assert_eq!(name_of(&log[0]), "5");
        assert_eq!(
            name_of(log.last().unwrap()),
            (MAX_UNDO_ACTIONS + 4).to_string()
        );
    }

    #[test]
    fn take_newest_takes_the_latest_match_and_misses_cleanly() {
        let (deck, card, other) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
        let mut store = UndoStore::default();
        store.push(deck, added(card, "first"));
        store.push(deck, added(other, "unrelated"));
        store.push(deck, added(card, "second"));
        let is_card =
            |a: &UndoAction| matches!(a, UndoAction::Added { card_id, .. } if *card_id == card);
        assert_eq!(
            name_of(&store.take_newest(deck, is_card).unwrap()),
            "second"
        );
        assert_eq!(name_of(&store.take_newest(deck, is_card).unwrap()), "first");
        assert!(store.take_newest(deck, is_card).is_none());
        assert_eq!(store.log(deck).len(), 1, "a miss takes nothing");
        assert!(store.take_newest(Uuid::new_v4(), is_card).is_none());
    }

    #[test]
    fn decks_keep_separate_logs_and_pop_newest_first() {
        let (a, b) = (Uuid::new_v4(), Uuid::new_v4());
        let mut store = UndoStore::default();
        store.push(a, added(Uuid::new_v4(), "a1"));
        store.push(a, added(Uuid::new_v4(), "a2"));
        assert!(store.is_empty(b));
        assert_eq!(name_of(&store.pop(a).unwrap()), "a2");
        store.clear(a);
        assert!(store.is_empty(a));
    }

    #[test]
    fn the_main_deck_is_the_servers_default_board() {
        assert_eq!(create_board_field(Board::Deck), None);
        assert_eq!(
            create_board_field(Board::Maybeboard).as_deref(),
            Some("maybeboard")
        );
        assert_eq!(
            create_board_field(Board::Sideboard).as_deref(),
            Some("sideboard")
        );
    }
}
