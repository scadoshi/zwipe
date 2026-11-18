//! In-memory undo log for deck mutations, walked by the deck cards screen.
//!
//! ONE per-deck global stack: every deck-mutation door records here (deck
//! cards, quick add, the add screen's swipes, the remove screen), and the
//! deck cards screen's Undo button pops and applies inverses newest-first
//! (see `apply_undo` in `view.rs`). The swipe screens' own gesture undo
//! stays local, but reconciles through this store: it takes its entry back
//! before reversing, and skips its server mutation when the entry is
//! already gone (consumed by the button). Memory-only by design: an undo
//! applies to this session's own changes.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::deck::{Board, DeckEntry};
use zwipe_ui::deck::command_zone::Slot;
use zwipe_ui::stores::undo::push_capped;

/// One undoable mutation, carrying what its inverse needs. Quantity bursts
/// coalesce through the debounce: one burst, one entry.
#[derive(Clone)]
pub(crate) enum UndoAction {
    /// A card was added (quick add or an add-screen swipe); undo deletes it
    /// (without the deliberate-removal telemetry signal: undoing an add is
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
        starred: Option<DateTime<Utc>>,
    },
    /// A card was starred or unstarred; undo puts its star back as `before` had it.
    MvpToggled {
        card_id: Uuid,
        card_name: String,
        before: Option<DateTime<Utc>>,
    },
    /// A card was flagged or unflagged; undo puts its flag back as `before` had it.
    FlagToggled {
        card_id: Uuid,
        card_name: String,
        before: bool,
    },
    /// A regular deck card changed printing; undo restores the old printing.
    PrintingChanged { old_card: Card, new_id: Uuid },
    /// A command-zone card changed printing: a deck-profile slot update,
    /// not a `deck_card` row; undo rebuilds the profile request with the old
    /// printing id and restores the slot's pinned card signal.
    CommandZonePrintingChanged {
        slot: Slot,
        old_card: Card,
        new_id: Uuid,
    },
    /// A card went to another deck through Send to, recorded on the deck it was sent from;
    /// undo takes the `copies` it sent back off `board` in `deck_id`.
    SentTo {
        card_id: Uuid,
        card_name: String,
        deck_id: Uuid,
        deck_name: String,
        board: Board,
        copies: i32,
    },
}

impl From<UndoAction> for zwipe_ui::stores::undo::UndoAction {
    fn from(action: UndoAction) -> Self {
        match action {
            UndoAction::Added { card_id, card_name } => Self::Added { card_id, card_name },
            UndoAction::Removed { entry, baseline } => Self::Removed {
                entry: Box::new(entry),
                baseline,
            },
            UndoAction::QuantityChanged {
                card_id,
                card_name,
                net,
                baseline,
            } => Self::QuantityChanged {
                card_id,
                card_name,
                net,
                baseline,
            },
            UndoAction::MovedBoard {
                card_id,
                card_name,
                from,
                starred,
            } => Self::MovedBoard {
                card_id,
                card_name,
                from,
                starred,
            },
            UndoAction::MvpToggled {
                card_id,
                card_name,
                before,
            } => Self::MvpToggled {
                card_id,
                card_name,
                before,
            },
            UndoAction::FlagToggled {
                card_id,
                card_name,
                before,
            } => Self::FlagToggled {
                card_id,
                card_name,
                before,
            },
            UndoAction::PrintingChanged { old_card, new_id } => Self::PrintingChanged {
                old_card: Box::new(old_card),
                new_id,
            },
            UndoAction::CommandZonePrintingChanged {
                slot,
                old_card,
                new_id,
            } => Self::CommandZonePrintingChanged {
                slot,
                old_card: Box::new(old_card),
                new_id,
            },
            UndoAction::SentTo {
                card_id,
                card_name,
                deck_id,
                deck_name,
                board,
                copies,
            } => Self::SentTo {
                card_id,
                card_name,
                deck_id,
                deck_name,
                board,
                copies,
            },
        }
    }
}

impl From<zwipe_ui::stores::undo::UndoAction> for UndoAction {
    fn from(action: zwipe_ui::stores::undo::UndoAction) -> Self {
        use zwipe_ui::stores::undo::UndoAction as Shared;
        match action {
            Shared::Added { card_id, card_name } => Self::Added { card_id, card_name },
            Shared::Removed { entry, baseline } => Self::Removed {
                entry: *entry,
                baseline,
            },
            Shared::QuantityChanged {
                card_id,
                card_name,
                net,
                baseline,
            } => Self::QuantityChanged {
                card_id,
                card_name,
                net,
                baseline,
            },
            Shared::MovedBoard {
                card_id,
                card_name,
                from,
                starred,
            } => Self::MovedBoard {
                card_id,
                card_name,
                from,
                starred,
            },
            Shared::MvpToggled {
                card_id,
                card_name,
                before,
            } => Self::MvpToggled {
                card_id,
                card_name,
                before,
            },
            Shared::FlagToggled {
                card_id,
                card_name,
                before,
            } => Self::FlagToggled {
                card_id,
                card_name,
                before,
            },
            Shared::PrintingChanged { old_card, new_id } => Self::PrintingChanged {
                old_card: *old_card,
                new_id,
            },
            Shared::CommandZonePrintingChanged {
                slot,
                old_card,
                new_id,
            } => Self::CommandZonePrintingChanged {
                slot,
                old_card: *old_card,
                new_id,
            },
            Shared::SentTo {
                card_id,
                card_name,
                deck_id,
                deck_name,
                board,
                copies,
            } => Self::SentTo {
                card_id,
                card_name,
                deck_id,
                deck_name,
                board,
                copies,
            },
        }
    }
}

/// Newest-first predicate removal: the reconciliation primitive behind
/// `UndoStore::take_newest`, split out so the selection logic is testable
/// without a signal runtime.
///
/// Entries carry no id: the swipe screens record an action before its server
/// call resolves, so a predicate over (variant, card) finds the entry instead.
pub(crate) fn take_newest_in(
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
    pub(crate) fn push(mut self, action: UndoAction) {
        push_capped(&mut self.0.write(), action);
    }
}

/// App-scoped park/restore for per-deck undo stacks, so the log survives
/// navigating away and back (mirroring `FilterStore`'s filter parking).
/// In-memory only: an app restart forgets. The stale-entry guards in
/// `apply_undo` cover the deck refetching fresher state between visits.
#[derive(Clone, Copy)]
pub(crate) struct UndoStore {
    entries: Signal<HashMap<Uuid, Vec<UndoAction>>>,
    /// The deck whose cards screen is open, and its live log.
    live: Signal<Option<(Uuid, UndoLog)>>,
}

/// Creates the store (provided as context in `spawn_upkeeper`, beside the
/// filter store).
pub(crate) fn use_undo_store() -> UndoStore {
    UndoStore {
        entries: use_signal(HashMap::new),
        live: use_signal(|| None),
    }
}

impl UndoStore {
    /// The stack last parked for this deck (empty if none). Clones rather
    /// than takes: restore runs inside the screen's mount initializer, and
    /// writing the store mid-render is the bug `FilterStore`'s clone-on-restore
    /// shape avoids; the leftover copy is simply overwritten on next park.
    pub(crate) fn restore(&self, deck_id: Uuid) -> Vec<UndoAction> {
        self.entries
            .peek()
            .get(&deck_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Remembers this deck's stack, replacing any prior one, and lets go of its live log.
    pub(crate) fn park(&mut self, deck_id: Uuid, log: Vec<UndoAction>) {
        if self.live.peek().is_some_and(|(open, _)| open == deck_id) {
            self.live.set(None);
        }
        self.entries.write().insert(deck_id, log);
    }

    /// Takes `log` as the open cards screen's live log for `deck_id`, so a change recorded
    /// from outside that screen (Send to) lands on it rather than under its parked copy.
    pub(crate) fn attach(&mut self, deck_id: Uuid, log: UndoLog) {
        self.live.set(Some((deck_id, log)));
    }

    /// Records `action` on `deck_id`'s history from outside its cards screen: on the live log
    /// while that screen is open, else on the parked stack.
    pub(crate) fn record(&mut self, deck_id: Uuid, action: UndoAction) {
        let live = *self.live.peek();
        match live {
            Some((open, log)) if open == deck_id => log.push(action),
            Some(_) | None => self.push(deck_id, action),
        }
    }

    /// Records a mutation made while the deck cards screen (and its live
    /// log) is unmounted: the add/remove screens' recording path. Writes
    /// straight into the parked stack; the next restore picks it up.
    pub(crate) fn push(&mut self, deck_id: Uuid, action: UndoAction) {
        push_capped(self.entries.write().entry(deck_id).or_default(), action);
    }

    /// Takes back the newest parked entry matching the predicate: gesture
    /// undo's reconciliation. `Some` means the mutation was still
    /// outstanding (proceed with the reversing server call); `None` means
    /// the button already consumed it (skip the server call, the world is
    /// already reversed). On a failed reversal, push the action back so the
    /// button's history stays truthful.
    pub(crate) fn take_newest(
        &mut self,
        deck_id: Uuid,
        matches: impl Fn(&UndoAction) -> bool,
    ) -> Option<UndoAction> {
        take_newest_in(self.entries.write().get_mut(&deck_id)?, matches)
    }

    /// Drops the deck's whole stack: import overwrites the deck, so every
    /// entry recorded before it is semantically void (the import screen's
    /// hint says so).
    pub(crate) fn clear(&mut self, deck_id: Uuid) {
        self.entries.write().remove(&deck_id);
    }
}

#[cfg(test)]
mod tests {
    use zwipe_ui::stores::undo::MAX_UNDO_ACTIONS;

    use super::*;

    fn added(card_id: Uuid, name: &str) -> UndoAction {
        UndoAction::Added {
            card_id,
            card_name: name.to_string(),
        }
    }

    fn name_of(action: &UndoAction) -> &str {
        if let UndoAction::Added { card_name, .. } = action {
            card_name
        } else {
            panic!("test log only holds Added")
        }
    }

    #[test]
    fn a_record_lands_on_the_open_screen_s_live_log_and_otherwise_parks() {
        thread_local! {
            /// What the store held at each step: the live log's length, then the parked
            /// stacks' lengths for the open deck and the other one.
            static SEEN: std::cell::RefCell<Vec<(usize, usize, usize)>> =
                const { std::cell::RefCell::new(Vec::new()) };
        }
        fn app() -> Element {
            let mut store = use_undo_store();
            let log = UndoLog(use_signal(Vec::new));
            use_hook(move || {
                let (open, other) = (Uuid::from_u128(1), Uuid::from_u128(2));
                let seen = move || {
                    let step = (
                        log.0.peek().len(),
                        store.restore(open).len(),
                        store.restore(other).len(),
                    );
                    SEEN.with_borrow_mut(|s| s.push(step));
                };
                store.attach(open, log);
                store.record(open, added(Uuid::new_v4(), "sent from the open deck"));
                store.record(other, added(Uuid::new_v4(), "sent from another"));
                seen();
                store.park(open, log.0.peek().clone());
                store.record(open, added(Uuid::new_v4(), "after it closed"));
                seen();
            });
            rsx! {}
        }
        VirtualDom::new(app).rebuild_in_place();
        let seen = SEEN.with_borrow(Clone::clone);
        assert_eq!(
            seen,
            [(1, 0, 1), (1, 2, 1)],
            "the open screen's log takes its deck's send and nothing parks under it; once it \
             closes, a send parks"
        );
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
        // The same card added twice with another card's entry between,
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
