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
    /// Per-`(commander, card)` keep/skip/maybe tallies from the add-card stack.
    signals: Mutex<HashMap<(Uuid, Uuid), CardTally>>,
    /// Per-candidate select/skip tallies from the Zwipe-select screen, keyed
    /// by the shown card's oracle id.
    select_signals: Mutex<HashMap<Uuid, SelectTally>>,
}

/// Add/skip/maybe/remove tally for one `(commander, card)` pair within a flush
/// window.
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

    /// Records one add-stack swipe as a `(commander, card)` suggestion signal.
    ///
    /// `Down` (undo) is ignored — only added/skipped/maybed express intent. No-op
    /// if either oracle id is missing (a deck with no commander, or a card with
    /// no oracle id, can't be attributed).
    pub fn record_signal(
        &self,
        commander_oracle_id: Option<Uuid>,
        card_oracle_id: Option<Uuid>,
        direction: Direction,
    ) {
        let (Some(commander), Some(card)) = (commander_oracle_id, card_oracle_id) else {
            return;
        };
        let Ok(mut map) = self.inner.signals.lock() else {
            return;
        };
        let tally = map.entry((commander, card)).or_default();
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

    /// Records one deliberate removal of a `(commander, card)` from a deck — a
    /// delayed negative signal, distinct from an add-stack skip. No-op if either
    /// oracle id is missing.
    pub fn record_removal(&self, commander_oracle_id: Option<Uuid>, card_oracle_id: Option<Uuid>) {
        let (Some(commander), Some(card)) = (commander_oracle_id, card_oracle_id) else {
            return;
        };
        if let Ok(mut map) = self.inner.signals.lock() {
            map.entry((commander, card)).or_default().removed += 1;
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
            .map(|((commander, card), t)| CardSignalDelta {
                commander_oracle_id: commander,
                card_oracle_id: card,
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
        let commander = Uuid::new_v4();
        let card = Uuid::new_v4();

        buffer.record_swipe(Direction::Left);
        buffer.record_signal(Some(commander), Some(card), Direction::Left);
        let batch = buffer.snapshot_and_zero().unwrap();
        assert_eq!(batch.swipes_left, 1);
        assert_eq!(batch.signals.len(), 1);
        assert!(batch.deck_skips.is_empty());

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
