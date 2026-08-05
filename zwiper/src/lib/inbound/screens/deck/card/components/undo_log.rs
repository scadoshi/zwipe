//! In-memory undo log for deck mutations, walked by the deck cards screen.
//!
//! ONE per-deck global stack: every deck-mutation door records here (deck
//! cards, quick add, the add screen's swipes, the remove screen), and the
//! deck cards screen's Undo button pops and applies inverses newest-first
//! (see `apply_undo` in `view.rs`). The swipe screens' own gesture undo
//! stays local, but reconciles through this store: it takes its entry back
//! before reversing, and skips its server mutation when the entry is
//! already gone (consumed by the button). Memory-only by design — the
//! decision trail is in `context/plans/global_undo.md` (built on
//! `archive/deck_cards_undo.md`).

use dioxus::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;
use zwipe_core::domain::{
    card::Card,
    deck::{Board, DeckEntry},
};

/// Cap — enough to walk back a whole editing session; oldest entries drop.
const MAX_UNDO_ACTIONS: usize = 100;

/// Identifies which command zone slot a card occupies for printing updates.
/// Lives here rather than in `view.rs`: the log is shared infrastructure,
/// and screens depend on it — never the reverse.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum CommandZoneSlot {
    Commander,
    Partner,
    Background,
    SignatureSpell,
}

/// One undoable mutation, carrying what its inverse needs. Quantity bursts
/// coalesce through the debounce: one burst, one entry.
#[derive(Clone)]
pub(crate) enum UndoAction {
    /// A card was added (quick add or an add-screen swipe); undo deletes it
    /// (without the deliberate-removal telemetry signal — undoing an add is
    /// not a removal opinion).
    Added { card_id: Uuid, card_name: String },
    /// A card was removed (qty crossed below 1, or a remove-screen swipe);
    /// undo re-creates it on its old board at its pre-burst quantity.
    Removed { entry: DeckEntry, baseline: i32 },
    /// A debounced quantity burst posted; undo applies the inverse net.
    QuantityChanged {
        card_id: Uuid,
        card_name: String,
        net: i32,
        baseline: i32,
    },
    /// A card moved boards; undo moves it back.
    MovedBoard {
        card_id: Uuid,
        card_name: String,
        from: Board,
    },
    /// A regular deck card changed printing; undo restores the old printing.
    PrintingChanged { old_card: Card, new_id: Uuid },
    /// A command-zone card changed printing — a deck-profile slot update,
    /// not a deck_card row; undo rebuilds the profile request with the old
    /// printing id and restores the slot's pinned card signal.
    CommandZonePrintingChanged {
        slot: CommandZoneSlot,
        old_card: Card,
        new_id: Uuid,
    },
}

/// Append with the cap enforced — shared by the live log and the parked
/// stacks so both drop oldest identically.
///
/// Entries carry no id: the plan sketched a per-entry u64, but the as-built
/// reconciliation (`take_newest` below) identifies an entry by matching its
/// action — the swipe screens record actions before their server call
/// resolves, so an id minted at record time had nothing to attach to, and a
/// predicate over (variant, card) picks the same entry newest-first.
fn push_capped(log: &mut Vec<UndoAction>, action: UndoAction) {
    log.push(action);
    if log.len() > MAX_UNDO_ACTIONS {
        let overflow = log.len() - MAX_UNDO_ACTIONS;
        log.drain(..overflow);
    }
}

/// Newest-first predicate removal — the reconciliation primitive behind
/// `UndoStore::take_newest`, split out so the selection logic is testable
/// without a signal runtime.
fn take_newest_in(
    log: &mut Vec<UndoAction>,
    matches: impl Fn(&UndoAction) -> bool,
) -> Option<UndoAction> {
    let index = log.iter().rposition(matches)?;
    Some(log.remove(index))
}

/// Context newtype so recorders inside the deck cards screen's tree (quick
/// add, the printing sheet) find the live log unambiguously.
#[derive(Clone, Copy)]
pub(crate) struct UndoLog(pub Signal<Vec<UndoAction>>);

impl UndoLog {
    /// Appends an action, dropping the oldest past the cap.
    pub fn push(mut self, action: UndoAction) {
        push_capped(&mut self.0.write(), action);
    }
}

/// App-scoped park/restore for per-deck undo stacks, so the log survives
/// navigating away and back (mirroring `FilterStore`'s filter parking —
/// filters surviving while undo evaporated read as a bug). In-memory only:
/// an app restart forgets. The stale-entry guards in `apply_undo` cover the
/// deck refetching fresher state between visits.
#[derive(Clone, Copy)]
pub(crate) struct UndoStore {
    entries: Signal<HashMap<Uuid, Vec<UndoAction>>>,
}

/// Creates the store (provided as context in `spawn_upkeeper`, beside the
/// filter store).
pub(crate) fn use_undo_store() -> UndoStore {
    UndoStore {
        entries: use_signal(HashMap::new),
    }
}

impl UndoStore {
    /// The stack last parked for this deck (empty if none). Clones rather
    /// than takes — restore runs inside the screen's mount initializer, and
    /// writing the store mid-render is the bug FilterStore's clone-on-restore
    /// shape avoids; the leftover copy is simply overwritten on next park.
    pub fn restore(&self, deck_id: Uuid) -> Vec<UndoAction> {
        self.entries
            .peek()
            .get(&deck_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Remembers this deck's stack, replacing any prior one.
    pub fn park(&mut self, deck_id: Uuid, log: Vec<UndoAction>) {
        self.entries.write().insert(deck_id, log);
    }

    /// Records a mutation made while the deck cards screen (and its live
    /// log) is unmounted — the add/remove screens' recording path. Writes
    /// straight into the parked stack; the next restore picks it up.
    pub fn push(&mut self, deck_id: Uuid, action: UndoAction) {
        push_capped(self.entries.write().entry(deck_id).or_default(), action);
    }

    /// Takes back the newest parked entry matching the predicate — gesture
    /// undo's reconciliation. `Some` means the mutation was still
    /// outstanding (proceed with the reversing server call); `None` means
    /// the button already consumed it (skip the server call, the world is
    /// already reversed). On a failed reversal, push the action back so the
    /// button's history stays truthful.
    pub fn take_newest(
        &mut self,
        deck_id: Uuid,
        matches: impl Fn(&UndoAction) -> bool,
    ) -> Option<UndoAction> {
        take_newest_in(self.entries.write().get_mut(&deck_id)?, matches)
    }

    /// Drops the deck's whole stack — import overwrites the deck, so every
    /// entry recorded before it is semantically void (owner decision in the
    /// plan; the import screen's hint discloses it).
    pub fn clear(&mut self, deck_id: Uuid) {
        self.entries.write().remove(&deck_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn added(card_id: Uuid, name: &str) -> UndoAction {
        UndoAction::Added {
            card_id,
            card_name: name.to_string(),
        }
    }

    fn name_of(action: &UndoAction) -> &str {
        match action {
            UndoAction::Added { card_name, .. } => card_name,
            _ => panic!("test log only holds Added"),
        }
    }

    #[test]
    fn push_capped_drops_oldest_past_the_cap() {
        let mut log = Vec::new();
        for i in 0..(MAX_UNDO_ACTIONS + 5) {
            push_capped(&mut log, added(Uuid::new_v4(), &i.to_string()));
        }
        assert_eq!(log.len(), MAX_UNDO_ACTIONS);
        // The five oldest fell off the bottom; the newest survived on top.
        assert_eq!(name_of(&log[0]), "5");
        assert_eq!(
            name_of(&log[log.len() - 1]),
            (MAX_UNDO_ACTIONS + 4).to_string()
        );
    }

    #[test]
    fn take_newest_picks_the_latest_match() {
        // The same card added twice with another card's entry between —
        // gesture undo of the second add must take the SECOND entry.
        let card = Uuid::new_v4();
        let other = Uuid::new_v4();
        let mut log = vec![
            added(card, "first add"),
            added(other, "unrelated"),
            added(card, "second add"),
        ];

        let taken = take_newest_in(
            &mut log,
            |a| matches!(a, UndoAction::Added { card_id, .. } if *card_id == card),
        )
        .expect("a match exists");
        assert_eq!(name_of(&taken), "second add");

        // A second take walks back to the earlier entry for the same card.
        let taken = take_newest_in(
            &mut log,
            |a| matches!(a, UndoAction::Added { card_id, .. } if *card_id == card),
        )
        .expect("the earlier entry remains");
        assert_eq!(name_of(&taken), "first add");

        // Only the unrelated entry survives.
        assert_eq!(log.len(), 1);
        assert_eq!(name_of(&log[0]), "unrelated");
    }

    #[test]
    fn take_newest_misses_cleanly() {
        let mut log = vec![added(Uuid::new_v4(), "only entry")];
        let taken = take_newest_in(&mut log, |_| false);
        assert!(taken.is_none());
        assert_eq!(log.len(), 1, "a miss must not consume anything");
    }
}
