//! In-memory swipe/search counters with snapshot-and-zero semantics.

use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU32, Ordering},
    },
};

use uuid::Uuid;
use zwipe_core::http::contracts::metrics::{CardSignalDelta, CommanderSelectDelta, HttpUsageBatch};

use crate::inbound::components::interactions::swipe::direction::Direction;

/// Signal buffer key: `(commander oracle id or None, card oracle id, deck id)`.
/// Commander is `None` for non-Commander decks; the deck id both distinguishes
/// otherwise-identical non-Commander keys and rides the wire so the server can
/// derive the generalized `(format, color-identity)` per-otag context.
type SignalKey = (Option<Uuid>, Uuid, Uuid);

/// Atomic buffer of pending usage counters. Cheap clone (Arc inner).
#[derive(Debug, Clone, Default)]
pub struct UsageBuffer {
    inner: Arc<UsageBufferInner>,
}

#[derive(Debug, Default)]
struct UsageBufferInner {
    swipes_right: AtomicU32,
    swipes_left: AtomicU32,
    swipes_up: AtomicU32,
    swipes_down: AtomicU32,
    searches: AtomicU32,
    /// Per-`(commander, card, deck)` keep/skip/maybe tallies from the add-card
    /// stack. Commander is `None` for non-Commander decks; the deck id lets the
    /// server derive the generalized `(format, color-identity)` per-otag context.
    signals: Mutex<HashMap<SignalKey, CardTally>>,
    /// Per-candidate select/skip tallies from the Zwipe-select screen, keyed
    /// by the shown card's oracle id.
    select_signals: Mutex<HashMap<Uuid, SelectTally>>,
}

/// Add/skip/maybe/remove tally for one `(commander, card, deck)` key within a
/// flush window.
#[derive(Debug, Default, Clone, Copy)]
struct CardTally {
    added: u32,
    skipped: u32,
    maybed: u32,
    removed: u32,
}

/// Select/skip tally for one command-zone candidate within a flush window.
#[derive(Debug, Default, Clone, Copy)]
struct SelectTally {
    selected: u32,
    skipped: u32,
}

impl UsageBuffer {
    /// Creates an empty buffer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Records one swipe in the given direction.
    pub fn record_swipe(&self, direction: Direction) {
        let counter = match direction {
            Direction::Right => &self.inner.swipes_right,
            Direction::Left => &self.inner.swipes_left,
            Direction::Up => &self.inner.swipes_up,
            Direction::Down => &self.inner.swipes_down,
        };
        counter.fetch_add(1, Ordering::Relaxed);
    }

    /// Records one card search.
    pub fn record_search(&self) {
        self.inner.searches.fetch_add(1, Ordering::Relaxed);
    }

    /// Records one add-stack swipe as a suggestion signal, keyed by
    /// `(commander, card, deck)`. Commander may be `None` (a non-Commander deck);
    /// the deck id still lets the server derive a generalized per-otag context, so
    /// non-Commander decks now contribute signal too.
    ///
    /// `Down` (undo) is ignored — only added/skipped/maybed express intent. No-op
    /// if the card has no oracle id.
    pub fn record_signal(
        &self,
        deck_id: Uuid,
        commander_oracle_id: Option<Uuid>,
        card_oracle_id: Option<Uuid>,
        direction: Direction,
    ) {
        let Some(card) = card_oracle_id else {
            return;
        };
        let Ok(mut map) = self.inner.signals.lock() else {
            return;
        };
        let tally = map.entry((commander_oracle_id, card, deck_id)).or_default();
        match direction {
            Direction::Right => tally.added += 1,
            Direction::Left => tally.skipped += 1,
            Direction::Up => tally.maybed += 1,
            Direction::Down => {}
        }
    }

    /// Records one Zwipe-select swipe as a per-candidate select signal, keyed
    /// by the shown card's oracle id.
    ///
    /// Only `Right` (selected) and `Left` (skipped) express a decision; `Down`
    /// (undo) and `Up` (unused on that screen) are ignored. No-op if the card
    /// has no oracle id.
    pub fn record_select_signal(&self, card_oracle_id: Option<Uuid>, direction: Direction) {
        let Some(card) = card_oracle_id else {
            return;
        };
        let Ok(mut map) = self.inner.select_signals.lock() else {
            return;
        };
        let tally = map.entry(card).or_default();
        match direction {
            Direction::Right => tally.selected += 1,
            Direction::Left => tally.skipped += 1,
            Direction::Up | Direction::Down => {}
        }
    }

    /// Records one deliberate removal of a card from a deck — a delayed negative
    /// signal, distinct from an add-stack skip. Keyed by `(commander, card, deck)`;
    /// commander may be `None`. No-op if the card has no oracle id.
    pub fn record_removal(
        &self,
        deck_id: Uuid,
        commander_oracle_id: Option<Uuid>,
        card_oracle_id: Option<Uuid>,
    ) {
        let Some(card) = card_oracle_id else {
            return;
        };
        if let Ok(mut map) = self.inner.signals.lock() {
            map.entry((commander_oracle_id, card, deck_id))
                .or_default()
                .removed += 1;
        }
    }

    /// Snapshots all counters into a batch and resets them to zero.
    ///
    /// Returns `None` when every counter is zero and no signals are buffered.
    pub fn snapshot_and_zero(&self) -> Option<HttpUsageBatch> {
        let right = self.inner.swipes_right.swap(0, Ordering::Relaxed);
        let left = self.inner.swipes_left.swap(0, Ordering::Relaxed);
        let up = self.inner.swipes_up.swap(0, Ordering::Relaxed);
        let down = self.inner.swipes_down.swap(0, Ordering::Relaxed);
        let searches = self.inner.searches.swap(0, Ordering::Relaxed);

        let signals: Vec<CardSignalDelta> = self
            .inner
            .signals
            .lock()
            .map(|mut map| std::mem::take(&mut *map))
            .unwrap_or_default()
            .into_iter()
            .map(|((_commander, card, deck_id), t)| CardSignalDelta {
                // Push `deck_id` only: the server derives the commander (EDH) or the
                // generalized `(format, color-identity)` context (non-EDH) from it.
                // The legacy `commander_oracle_id` wire field is left `None` and is
                // sunset once 1.6.1 is the min-version floor.
                commander_oracle_id: None,
                card_oracle_id: card,
                // The deck the swipes belong to. Lets the server derive the
                // richer generalized-context per-otag signal, and (for
                // non-Commander decks) is the sole context key.
                deck_id: Some(deck_id),
                // `shown` is derived from add-stack actions for now; a removal is
                // not an impression, so it doesn't count toward `shown`.
                shown: t.added + t.skipped + t.maybed,
                added: t.added,
                skipped: t.skipped,
                maybed: t.maybed,
                removed: t.removed,
            })
            .collect();

        let select_signals: Vec<CommanderSelectDelta> = self
            .inner
            .select_signals
            .lock()
            .map(|mut map| std::mem::take(&mut *map))
            .unwrap_or_default()
            .into_iter()
            .map(|(card, t)| CommanderSelectDelta {
                commander_oracle_id: card,
                shown: t.selected + t.skipped,
                selected: t.selected,
                skipped: t.skipped,
            })
            .collect();

        if right == 0
            && left == 0
            && up == 0
            && down == 0
            && searches == 0
            && signals.is_empty()
            && select_signals.is_empty()
        {
            return None;
        }

        Some(HttpUsageBatch {
            swipes_right: right,
            swipes_left: left,
            swipes_up: up,
            swipes_down: down,
            searches,
            // Durable skips post directly per swipe (skip_deck_card); the
            // batch field remains for wire compat with the server.
            deck_skips: Vec::new(),
            signals,
            select_signals,
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::UsageBuffer;
    use crate::inbound::components::interactions::swipe::direction::Direction;
    use uuid::Uuid;

    #[test]
    fn snapshot_drains_once_and_resets() {
        let buffer = UsageBuffer::new();
        let deck = Uuid::new_v4();
        let commander = Uuid::new_v4();
        let card = Uuid::new_v4();

        buffer.record_swipe(Direction::Left);
        buffer.record_signal(deck, Some(commander), Some(card), Direction::Left);
        let batch = buffer.snapshot_and_zero().unwrap();
        assert_eq!(batch.swipes_left, 1);
        assert_eq!(batch.signals.len(), 1);
        let delta = batch.signals.first().unwrap();
        // The client pushes deck_id only now; commander is derived server-side.
        assert_eq!(delta.commander_oracle_id, None);
        assert_eq!(delta.deck_id, Some(deck));
        assert!(batch.deck_skips.is_empty());

        assert!(buffer.snapshot_and_zero().is_none());
    }

    #[test]
    fn non_commander_deck_still_signals_with_deck_context() {
        let buffer = UsageBuffer::new();
        let deck = Uuid::new_v4();
        let card = Uuid::new_v4();

        // No commander (non-Commander deck): the signal must still be recorded,
        // carrying the deck id so the server can derive its (format, CI) context.
        buffer.record_signal(deck, None, Some(card), Direction::Right);
        let batch = buffer.snapshot_and_zero().unwrap();
        assert_eq!(batch.signals.len(), 1);
        let delta = batch.signals.first().unwrap();
        assert_eq!(delta.commander_oracle_id, None);
        assert_eq!(delta.deck_id, Some(deck));
        assert_eq!(delta.card_oracle_id, card);
        assert_eq!(delta.added, 1);
    }

    #[test]
    fn signal_with_no_card_oracle_id_is_dropped() {
        let buffer = UsageBuffer::new();
        let deck = Uuid::new_v4();
        buffer.record_signal(deck, None, None, Direction::Right);
        assert!(buffer.snapshot_and_zero().is_none());
    }

    #[test]
    fn select_signal_tallies_and_derives_shown() {
        let buffer = UsageBuffer::new();
        let card = Uuid::new_v4();

        buffer.record_select_signal(Some(card), Direction::Left);
        buffer.record_select_signal(Some(card), Direction::Left);
        buffer.record_select_signal(Some(card), Direction::Right);
        // Undo and missing-oracle-id swipes contribute nothing.
        buffer.record_select_signal(Some(card), Direction::Down);
        buffer.record_select_signal(None, Direction::Right);

        let batch = buffer.snapshot_and_zero().unwrap();
        assert_eq!(batch.select_signals.len(), 1);
        let delta = batch.select_signals.first().unwrap();
        assert_eq!(delta.commander_oracle_id, card);
        assert_eq!(delta.selected, 1);
        assert_eq!(delta.skipped, 2);
        assert_eq!(delta.shown, 3);

        assert!(buffer.snapshot_and_zero().is_none());
    }
}
