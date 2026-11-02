//! The usage counted between flushes, drained into one batch at a time.

use std::collections::BTreeMap;
use std::mem;

use uuid::Uuid;
use zwipe_core::domain::auth::models::platform::ClientPlatform;
use zwipe_core::http::contracts::metrics::{
    CardSignalDelta, ClientErrorReport, CommanderSelectDelta, HttpUsageBatch, PileEndKind,
    PileEndReport,
};

use super::handled_error::HandledError;

/// The way a card left a swipe surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SwipeDirection {
    /// Left: skipped.
    Left,
    /// Right: added or selected.
    Right,
    /// Up: maybe.
    Up,
    /// Down: undo.
    Down,
}

/// Add, skip, maybe and remove tallies for one `(card, deck)` within a flush window.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct CardTally {
    added: u32,
    skipped: u32,
    maybed: u32,
    removed: u32,
}

/// Select and skip tallies for one command-zone candidate within a flush window.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct SelectTally {
    selected: u32,
    skipped: u32,
}

/// Swipe and search counts within a flush window.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct Counts {
    right: u32,
    left: u32,
    up: u32,
    down: u32,
    searches: u32,
}

/// The usage a client has counted since its last flush.
///
/// Recording takes `&mut self`; a client that records from several tasks wraps the buffer in its own lock. [`snapshot_and_zero`](Self::snapshot_and_zero) drains everything into one [`HttpUsageBatch`], tagging each error report with the platform and version the buffer was built with. Counts saturate instead of wrapping, handled errors stop growing at [`HttpUsageBatch::MAX_CLIENT_ERRORS_PER_FLUSH`] distinct entries and pile ends at [`HttpUsageBatch::MAX_PILE_ENDS_PER_FLUSH`], and a batch lists its entries in a fixed order, so the same input always makes the same batch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageBuffer {
    platform: ClientPlatform,
    client_version: &'static str,
    counts: Counts,
    /// Keyed by `(card oracle id, deck id)`. The deck id is the only context sent: the server derives the commander, or the format and color identity, from it.
    signals: BTreeMap<(Uuid, Uuid), CardTally>,
    /// Keyed by the shown candidate's oracle id.
    select_signals: BTreeMap<Uuid, SelectTally>,
    errors: BTreeMap<HandledError, u32>,
    pile_ends: Vec<PileEndReport>,
}

impl UsageBuffer {
    /// An empty buffer whose error reports name `platform` and `client_version`, the sending build's `CARGO_PKG_VERSION`.
    #[must_use]
    pub fn new(platform: ClientPlatform, client_version: &'static str) -> Self {
        Self {
            platform,
            client_version,
            counts: Counts::default(),
            signals: BTreeMap::new(),
            select_signals: BTreeMap::new(),
            errors: BTreeMap::new(),
            pile_ends: Vec::new(),
        }
    }

    /// Counts one swipe on any swipe surface.
    pub fn record_swipe(&mut self, direction: SwipeDirection) {
        let counter = match direction {
            SwipeDirection::Right => &mut self.counts.right,
            SwipeDirection::Left => &mut self.counts.left,
            SwipeDirection::Up => &mut self.counts.up,
            SwipeDirection::Down => &mut self.counts.down,
        };
        *counter = counter.saturating_add(1);
    }

    /// Counts one card search.
    pub fn record_search(&mut self) {
        self.counts.searches = self.counts.searches.saturating_add(1);
    }

    /// Counts one Add-stack swipe as a suggestion signal for `(card, deck)`.
    ///
    /// Right adds, Left skips and Up maybes; Down (undo) counts nothing. A card with no oracle id counts nothing.
    pub fn record_signal(
        &mut self,
        deck_id: Uuid,
        card_oracle_id: Option<Uuid>,
        direction: SwipeDirection,
    ) {
        let Some(card) = card_oracle_id else {
            return;
        };
        let tally = self.signals.entry((card, deck_id)).or_default();
        match direction {
            SwipeDirection::Right => tally.added = tally.added.saturating_add(1),
            SwipeDirection::Left => tally.skipped = tally.skipped.saturating_add(1),
            SwipeDirection::Up => tally.maybed = tally.maybed.saturating_add(1),
            SwipeDirection::Down => {}
        }
    }

    /// Counts one swipe select decision for the shown candidate.
    ///
    /// Right selects and Left skips. Up (a commander maybeboard save, not a pick) and Down (undo) count nothing, nor does a card with no oracle id.
    pub fn record_select_signal(
        &mut self,
        card_oracle_id: Option<Uuid>,
        direction: SwipeDirection,
    ) {
        let Some(card) = card_oracle_id else {
            return;
        };
        let tally = self.select_signals.entry(card).or_default();
        match direction {
            SwipeDirection::Right => tally.selected = tally.selected.saturating_add(1),
            SwipeDirection::Left => tally.skipped = tally.skipped.saturating_add(1),
            SwipeDirection::Up | SwipeDirection::Down => {}
        }
    }

    /// Counts one deliberate removal of a card from a deck: a late negative signal, apart from an Add-stack skip. A card with no oracle id counts nothing.
    pub fn record_removal(&mut self, deck_id: Uuid, card_oracle_id: Option<Uuid>) {
        let Some(card) = card_oracle_id else {
            return;
        };
        let tally = self.signals.entry((card, deck_id)).or_default();
        tally.removed = tally.removed.saturating_add(1);
    }

    /// Counts one time a deck's Add pile ran out.
    pub fn record_pile_end(&mut self, deck_id: Uuid, kind: PileEndKind, synergy: bool) {
        if self.pile_ends.len() < HttpUsageBatch::MAX_PILE_ENDS_PER_FLUSH {
            self.pile_ends.push(PileEndReport {
                deck_id,
                kind,
                synergy,
            });
        }
    }

    /// Counts one handled error the user just saw. A repeat raises the existing entry's count; a new error past the cap is dropped.
    pub fn report_error(&mut self, error: HandledError) {
        if let Some(count) = self.errors.get_mut(&error) {
            *count = count.saturating_add(1);
        } else if self.errors.len() < HttpUsageBatch::MAX_CLIENT_ERRORS_PER_FLUSH {
            self.errors.insert(error, 1);
        }
    }

    /// Drops everything counted without sending it, as when the session ends.
    pub fn clear(&mut self) {
        drop(self.snapshot_and_zero());
    }

    /// Drains everything counted into one batch, or `None` when nothing was counted.
    ///
    /// A signal's `shown` is its adds, skips and maybes; a removal is not a showing. A tally whose `shown` would pass `u32::MAX` is left out rather than sent wrong.
    #[must_use = "the batch is drained; drop it with clear"]
    pub fn snapshot_and_zero(&mut self) -> Option<HttpUsageBatch> {
        let counts = mem::take(&mut self.counts);

        let signals: Vec<CardSignalDelta> = mem::take(&mut self.signals)
            .into_iter()
            .filter_map(|((card, deck_id), t)| {
                let Some(shown) = t
                    .added
                    .checked_add(t.skipped)
                    .and_then(|n| n.checked_add(t.maybed))
                else {
                    tracing::warn!(%card, %deck_id, "signal tally overflows u32, dropped");
                    return None;
                };
                Some(CardSignalDelta {
                    card_oracle_id: card,
                    deck_id: Some(deck_id),
                    shown,
                    added: t.added,
                    skipped: t.skipped,
                    maybed: t.maybed,
                    removed: t.removed,
                })
            })
            .collect();

        let select_signals: Vec<CommanderSelectDelta> = mem::take(&mut self.select_signals)
            .into_iter()
            .filter_map(|(card, t)| {
                let Some(shown) = t.selected.checked_add(t.skipped) else {
                    tracing::warn!(%card, "select tally overflows u32, dropped");
                    return None;
                };
                Some(CommanderSelectDelta {
                    commander_oracle_id: card,
                    shown,
                    selected: t.selected,
                    skipped: t.skipped,
                })
            })
            .collect();

        let platform = self.platform.as_str();
        let client_errors: Vec<ClientErrorReport> = mem::take(&mut self.errors)
            .into_iter()
            .map(|(error, count)| error.into_report(count, self.client_version, platform))
            .collect();

        let pile_ends = mem::take(&mut self.pile_ends);

        if counts == Counts::default()
            && signals.is_empty()
            && select_signals.is_empty()
            && client_errors.is_empty()
            && pile_ends.is_empty()
        {
            return None;
        }

        Some(HttpUsageBatch {
            swipes_right: counts.right,
            swipes_left: counts.left,
            swipes_up: counts.up,
            swipes_down: counts.down,
            searches: counts.searches,
            // Skips post one by one as they happen; the field stays for the wire format.
            deck_skips: Vec::new(),
            signals,
            select_signals,
            client_errors,
            pile_ends,
        })
    }
}

#[cfg(test)]
mod tests {
    use zwipe_core::domain::auth::models::platform::ClientPlatform;
    use zwipe_core::http::contracts::metrics::{HttpUsageBatch, PileEndKind, PileEndReport};

    use super::*;
    use crate::telemetry::ErrorKind;

    const VERSION: &str = "9.9.9";

    fn buffer() -> UsageBuffer {
        UsageBuffer::new(ClientPlatform::Ios, VERSION)
    }

    fn unprocessable(action: &'static str) -> HandledError {
        HandledError::new(
            "deck_edit",
            "",
            action,
            ErrorKind::Unprocessable,
            "deck limit reached",
        )
    }

    #[test]
    fn snapshot_drains_once_and_resets() {
        let mut buffer = buffer();
        let deck = Uuid::new_v4();
        let card = Uuid::new_v4();

        buffer.record_swipe(SwipeDirection::Left);
        buffer.record_signal(deck, Some(card), SwipeDirection::Left);
        let batch = buffer.snapshot_and_zero().unwrap();
        assert_eq!(batch.swipes_left, 1);
        assert_eq!(batch.signals.len(), 1);
        let delta = batch.signals.first().unwrap();
        // The client sends deck_id only; the server derives all context.
        assert_eq!(delta.deck_id, Some(deck));
        assert_eq!(batch.deck_skips, Vec::new());

        assert!(buffer.snapshot_and_zero().is_none());
    }

    #[test]
    fn every_direction_counts_its_own_swipes() {
        let mut buffer = buffer();
        for direction in [
            SwipeDirection::Right,
            SwipeDirection::Right,
            SwipeDirection::Left,
            SwipeDirection::Up,
            SwipeDirection::Up,
            SwipeDirection::Up,
            SwipeDirection::Down,
        ] {
            buffer.record_swipe(direction);
        }
        buffer.record_search();
        buffer.record_search();
        let batch = buffer.snapshot_and_zero().unwrap();
        assert_eq!(
            (
                batch.swipes_right,
                batch.swipes_left,
                batch.swipes_up,
                batch.swipes_down,
                batch.searches
            ),
            (2, 1, 3, 1, 2)
        );
    }

    #[test]
    fn a_full_tally_stays_full() {
        let mut buffer = buffer();
        let (deck, card) = (Uuid::new_v4(), Uuid::new_v4());
        buffer.signals.insert(
            (card, deck),
            CardTally {
                added: u32::MAX,
                ..CardTally::default()
            },
        );
        buffer.record_signal(deck, Some(card), SwipeDirection::Right);
        assert_eq!(buffer.signals[&(card, deck)].added, u32::MAX);
    }

    #[test]
    fn a_full_swipe_count_stays_full() {
        let mut buffer = buffer();
        buffer.counts.right = u32::MAX;
        buffer.record_swipe(SwipeDirection::Right);
        assert_eq!(buffer.counts.right, u32::MAX);
    }

    #[test]
    fn a_shown_count_past_u32_drops_its_delta() {
        let mut buffer = buffer();
        let (deck, card) = (Uuid::new_v4(), Uuid::new_v4());
        buffer.signals.insert(
            (card, deck),
            CardTally {
                added: u32::MAX,
                skipped: 1,
                ..CardTally::default()
            },
        );
        buffer.select_signals.insert(
            card,
            SelectTally {
                selected: u32::MAX,
                skipped: 1,
            },
        );
        buffer.record_search();
        let batch = buffer.snapshot_and_zero().unwrap();
        assert!(batch.signals.is_empty(), "{:?}", batch.signals);
        assert!(
            batch.select_signals.is_empty(),
            "{:?}",
            batch.select_signals
        );
        assert_eq!(batch.searches, 1);
    }

    #[test]
    fn a_pile_end_alone_makes_a_batch_and_drains() {
        let mut buffer = buffer();
        let deck = Uuid::new_v4();

        buffer.record_pile_end(deck, PileEndKind::Empty, true);
        let batch = buffer.snapshot_and_zero().unwrap();
        assert_eq!(
            batch.pile_ends,
            [PileEndReport {
                deck_id: deck,
                kind: PileEndKind::Empty,
                synergy: true,
            }]
        );
        assert!(buffer.snapshot_and_zero().is_none());
    }

    #[test]
    fn pile_ends_stop_at_the_cap() {
        let mut buffer = buffer();
        let deck = Uuid::new_v4();
        for _ in 0..(HttpUsageBatch::MAX_PILE_ENDS_PER_FLUSH + 3) {
            buffer.record_pile_end(deck, PileEndKind::Empty, false);
        }
        let batch = buffer.snapshot_and_zero().unwrap();
        assert_eq!(
            batch.pile_ends.len(),
            HttpUsageBatch::MAX_PILE_ENDS_PER_FLUSH
        );
    }

    #[test]
    fn signal_carries_deck_context() {
        let mut buffer = buffer();
        let deck = Uuid::new_v4();
        let card = Uuid::new_v4();

        buffer.record_signal(deck, Some(card), SwipeDirection::Right);
        let batch = buffer.snapshot_and_zero().unwrap();
        assert_eq!(batch.signals.len(), 1);
        let delta = batch.signals.first().unwrap();
        assert_eq!(delta.deck_id, Some(deck));
        assert_eq!(delta.card_oracle_id, card);
        assert_eq!(delta.added, 1);
    }

    #[test]
    fn signal_with_no_card_oracle_id_is_dropped() {
        let mut buffer = buffer();
        let deck = Uuid::new_v4();
        buffer.record_signal(deck, None, SwipeDirection::Right);
        buffer.record_removal(deck, None);
        assert!(buffer.snapshot_and_zero().is_none());
    }

    #[test]
    fn signals_tally_and_removals_are_not_shown() {
        let mut buffer = buffer();
        let (deck, card) = (Uuid::new_v4(), Uuid::new_v4());
        buffer.record_signal(deck, Some(card), SwipeDirection::Right);
        buffer.record_signal(deck, Some(card), SwipeDirection::Left);
        buffer.record_signal(deck, Some(card), SwipeDirection::Up);
        buffer.record_signal(deck, Some(card), SwipeDirection::Down);
        buffer.record_removal(deck, Some(card));
        let batch = buffer.snapshot_and_zero().unwrap();
        assert_eq!(
            batch.signals,
            [CardSignalDelta {
                card_oracle_id: card,
                deck_id: Some(deck),
                shown: 3,
                added: 1,
                skipped: 1,
                maybed: 1,
                removed: 1,
            }]
        );
        assert_eq!(batch.swipes_down, 0, "a signal is not a swipe count");
    }

    #[test]
    fn select_signal_tallies_and_derives_shown() {
        let mut buffer = buffer();
        let card = Uuid::new_v4();

        buffer.record_select_signal(Some(card), SwipeDirection::Left);
        buffer.record_select_signal(Some(card), SwipeDirection::Left);
        buffer.record_select_signal(Some(card), SwipeDirection::Right);
        // Undo, maybeboard saves and missing-oracle-id swipes contribute nothing.
        buffer.record_select_signal(Some(card), SwipeDirection::Down);
        buffer.record_select_signal(Some(card), SwipeDirection::Up);
        buffer.record_select_signal(None, SwipeDirection::Right);

        let batch = buffer.snapshot_and_zero().unwrap();
        assert_eq!(batch.select_signals.len(), 1);
        let delta = batch.select_signals.first().unwrap();
        assert_eq!(delta.commander_oracle_id, card);
        assert_eq!(delta.selected, 1);
        assert_eq!(delta.skipped, 2);
        assert_eq!(delta.shown, 3);

        assert!(buffer.snapshot_and_zero().is_none());
    }

    #[test]
    fn report_error_dedupes_and_caps() {
        let mut buffer = buffer();

        // The same error three times makes one entry with count 3.
        for _ in 0..3 {
            buffer.report_error(unprocessable("save_profile"));
        }
        // Distinct actions beyond the cap are dropped, not accumulated.
        for i in 0..(HttpUsageBatch::MAX_CLIENT_ERRORS_PER_FLUSH + 10) {
            let action: &'static str = Box::leak(format!("action_{i}").into_boxed_str());
            buffer.report_error(unprocessable(action));
        }

        let batch = buffer.snapshot_and_zero().expect("errors buffered");
        assert_eq!(
            batch.client_errors.len(),
            HttpUsageBatch::MAX_CLIENT_ERRORS_PER_FLUSH,
            "distinct entries capped"
        );
        let repeated = batch
            .client_errors
            .iter()
            .find(|r| r.action == "save_profile")
            .expect("deduped entry present");
        assert_eq!(repeated.count, 3);
        assert_eq!(repeated.kind, "api_unprocessable");
        assert_eq!(repeated.message, "deck limit reached");

        assert!(buffer.snapshot_and_zero().is_none());
    }

    #[test]
    fn a_report_names_the_buffers_platform_and_version() {
        let mut buffer = UsageBuffer::new(ClientPlatform::Terminal, VERSION);
        buffer.report_error(unprocessable("save"));
        let batch = buffer.snapshot_and_zero().unwrap();
        let report = batch.client_errors.first().unwrap();
        assert_eq!(report.platform, "terminal");
        assert_eq!(report.client_version, VERSION);
        assert_eq!(report.screen, "deck_edit");
        assert_eq!(report.component, "");
        assert_eq!(
            (
                batch.swipes_right,
                batch.swipes_left,
                batch.swipes_up,
                batch.swipes_down,
                batch.searches
            ),
            (0, 0, 0, 0, 0),
            "an error alone counts no usage"
        );
    }

    #[test]
    fn the_same_input_makes_the_same_batch() {
        let decks = [Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
        let cards = [Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
        let record = |order: &[usize]| {
            let mut buffer = buffer();
            for &i in order {
                buffer.record_signal(decks[i], Some(cards[i]), SwipeDirection::Right);
                buffer.record_select_signal(Some(cards[i]), SwipeDirection::Left);
                buffer.report_error(unprocessable(["a", "b", "c"][i]));
            }
            buffer.snapshot_and_zero().unwrap()
        };
        assert_eq!(record(&[0, 1, 2]), record(&[2, 0, 1]));
    }

    #[test]
    fn clear_drops_everything() {
        let mut buffer = buffer();
        buffer.record_swipe(SwipeDirection::Right);
        buffer.record_search();
        buffer.record_removal(Uuid::new_v4(), Some(Uuid::new_v4()));
        buffer.report_error(unprocessable("save"));
        buffer.clear();
        assert!(buffer.snapshot_and_zero().is_none());
        assert_eq!(buffer, UsageBuffer::new(ClientPlatform::Ios, VERSION));
    }
}
