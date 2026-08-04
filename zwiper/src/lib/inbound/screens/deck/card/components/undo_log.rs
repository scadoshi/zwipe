//! In-memory undo log for the deck cards screen.
//!
//! Each mutation on the screen pushes its inverse; the ActionBar's
//! conditional Undo button pops and applies them newest-first (see
//! `apply_undo` in `view.rs`). Screen-scoped and memory-only by design —
//! the full decision trail (no persistence, no list UI, the v1 action set)
//! is in `context/plans/deck_cards_undo.md`.

use dioxus::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;
use zwipe_core::domain::{
    card::Card,
    deck::{Board, DeckEntry},
};

/// Cap — enough to walk back a whole editing session; oldest entries drop.
const MAX_UNDO_ACTIONS: usize = 100;

/// One undoable mutation, carrying what its inverse needs. Quantity bursts
/// coalesce through the debounce: one burst, one entry.
#[derive(Clone)]
pub(crate) enum UndoAction {
    /// Quick-add added a card; undo deletes it (without the deliberate-removal
    /// telemetry signal — undoing an add is not a removal opinion).
    Added { card_id: Uuid, card_name: String },
    /// A card was removed (qty crossed below 1); undo re-creates it on its
    /// old board at its pre-burst quantity.
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
    /// (Command-zone printings are out of scope — see the plan.)
    PrintingChanged { old_card: Card, new_id: Uuid },
}

/// Context newtype so recorders outside `view.rs` (quick add) find the
/// screen's log unambiguously.
#[derive(Clone, Copy)]
pub(crate) struct UndoLog(pub Signal<Vec<UndoAction>>);

impl UndoLog {
    /// Appends an action, dropping the oldest past the cap.
    pub fn push(mut self, action: UndoAction) {
        let mut log = self.0.write();
        log.push(action);
        if log.len() > MAX_UNDO_ACTIONS {
            let overflow = log.len() - MAX_UNDO_ACTIONS;
            log.drain(..overflow);
        }
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
}
