//! In-memory swipe/search counters with snapshot-and-zero semantics.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use uuid::Uuid;
use zwipe_client::ClientError;
use zwipe_core::domain::auth::models::platform::ClientPlatform;
use zwipe_core::http::contracts::metrics::{
    CardSignalDelta, ClientErrorReport, CommanderSelectDelta, HttpUsageBatch, PileEndKind,
    PileEndReport,
};

use crate::inbound::components::interactions::swipe::direction::Direction;

/// Signal buffer key: `(card oracle id, deck id)`. The deck id rides the wire
/// as the sole context key: the server derives the commander (EDH) or the
/// generalized `(format, color-identity)` per-otag context from it.
type SignalKey = (Uuid, Uuid);

/// Handled-error dedupe key within a flush window. Four same-typed label
/// fields make a tuple a positional-swap hazard; named fields keep every
/// construction site honest.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ErrorKey {
    screen: &'static str,
    component: &'static str,
    action: &'static str,
    kind: &'static str,
    message: String,
}

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
    /// Per-`(card, deck)` keep/skip/maybe tallies from the add-card stack. The
    /// deck id lets the server derive the commander or the generalized
    /// `(format, color-identity)` per-otag context.
    signals: Mutex<HashMap<SignalKey, CardTally>>,
    /// Per-candidate select/skip tallies from the Zwipe-select screen, keyed
    /// by the shown card's oracle id.
    select_signals: Mutex<HashMap<Uuid, SelectTally>>,
    /// Handled-error reports deduped by [`ErrorKey`]. Capped at
    /// [`HttpUsageBatch::MAX_CLIENT_ERRORS_PER_FLUSH`] distinct entries per
    /// flush window: an error loop increments existing counts but never
    /// grows the map past the cap.
    errors: Mutex<HashMap<ErrorKey, u32>>,
    /// Times a deck's add-screen pile ran out, capped at
    /// [`HttpUsageBatch::MAX_PILE_ENDS_PER_FLUSH`] per flush window.
    pile_ends: Mutex<Vec<PileEndReport>>,
}

/// Add/skip/maybe/remove tally for one `(card, deck)` key within a flush
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

    /// Records one add-stack swipe as a suggestion signal, keyed by
    /// `(card, deck)`. The server derives the deck's commander or generalized
    /// per-otag context, so non-Commander decks contribute signal too.
    ///
    /// `Down` (undo) is ignored: only added/skipped/maybed express intent. No-op
    /// if the card has no oracle id.
    pub fn record_signal(&self, deck_id: Uuid, card_oracle_id: Option<Uuid>, direction: Direction) {
        let Some(card) = card_oracle_id else {
            return;
        };
        let Ok(mut map) = self.inner.signals.lock() else {
            return;
        };
        let tally = map.entry((card, deck_id)).or_default();
        // A full tally stays full, like the error count in `report_error`.
        match direction {
            Direction::Right => tally.added = tally.added.saturating_add(1),
            Direction::Left => tally.skipped = tally.skipped.saturating_add(1),
            Direction::Up => tally.maybed = tally.maybed.saturating_add(1),
            Direction::Down => {}
        }
    }

    /// Records one Zwipe-select swipe as a per-candidate select signal, keyed
    /// by the shown card's oracle id.
    ///
    /// Only `Right` (selected) and `Left` (skipped) express a decision; `Down`
    /// (undo) and `Up` (commander maybeboard save: non-committal, not a
    /// pick) are ignored. No-op if the card has no oracle id.
    pub fn record_select_signal(&self, card_oracle_id: Option<Uuid>, direction: Direction) {
        let Some(card) = card_oracle_id else {
            return;
        };
        let Ok(mut map) = self.inner.select_signals.lock() else {
            return;
        };
        let tally = map.entry(card).or_default();
        // A full tally stays full, as in `record_signal`.
        match direction {
            Direction::Right => tally.selected = tally.selected.saturating_add(1),
            Direction::Left => tally.skipped = tally.skipped.saturating_add(1),
            Direction::Up | Direction::Down => {}
        }
    }

    /// Reports a handled error that was just surfaced to the user, and mirrors
    /// it into the local tracing log (one call, two sinks: client logs and
    /// the server table tell the same story).
    ///
    /// `screen`/`component`/`action` come from [`super::vocabulary`], the
    /// closed, module-path-derived breadcrumb. `component` is `""` when the
    /// screen itself surfaced the error.
    ///
    /// `Network` errors are logged but never buffered: they're mostly the
    /// user's connectivity, and couldn't be reported over the same dead
    /// connection anyway. `Decode` messages are reduced to the error's shape
    /// (quoted input fragments stripped): response bodies must never ride a
    /// report or a log line.
    pub fn report_error(
        &self,
        screen: &'static str,
        component: &'static str,
        action: &'static str,
        error: &ClientError,
    ) {
        let (kind, message) = match error {
            ClientError::Network(_) => {
                tracing::warn!(
                    screen,
                    component,
                    action,
                    kind = "network",
                    "error shown: {error}"
                );
                return;
            }
            ClientError::Decode(detail) => {
                ("decode", sanitized_decode_message(&detail.to_string()))
            }
            api => (api_error_kind(api), api.to_string()),
        };
        let message: String = message
            .chars()
            .take(ClientErrorReport::MAX_MESSAGE_CHARS)
            .collect();
        tracing::warn!(screen, component, action, kind, "error shown: {message}");

        let Ok(mut map) = self.inner.errors.lock() else {
            return;
        };
        let key = ErrorKey {
            screen,
            component,
            action,
            kind,
            message,
        };
        if let Some(count) = map.get_mut(&key) {
            *count = count.saturating_add(1);
        } else if map.len() < HttpUsageBatch::MAX_CLIENT_ERRORS_PER_FLUSH {
            map.insert(key, 1);
        }
    }

    /// Records one deliberate removal of a card from a deck: a delayed negative
    /// signal, distinct from an add-stack skip. Keyed by `(card, deck)`. No-op
    /// if the card has no oracle id.
    pub fn record_removal(&self, deck_id: Uuid, card_oracle_id: Option<Uuid>) {
        let Some(card) = card_oracle_id else {
            return;
        };
        if let Ok(mut map) = self.inner.signals.lock() {
            let tally = map.entry((card, deck_id)).or_default();
            // A full tally stays full, as in `record_signal`.
            tally.removed = tally.removed.saturating_add(1);
        }
    }

    /// Records one time a deck's add-screen pile ran out.
    pub fn record_pile_end(&self, deck_id: Uuid, kind: PileEndKind, synergy: bool) {
        if let Ok(mut ends) = self.inner.pile_ends.lock()
            && ends.len() < HttpUsageBatch::MAX_PILE_ENDS_PER_FLUSH
        {
            ends.push(PileEndReport {
                deck_id,
                kind,
                synergy,
            });
        }
    }

    /// Drops everything buffered without sending it.
    pub fn clear(&self) {
        drop(self.snapshot_and_zero());
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
            .filter_map(|((card, deck_id), t)| {
                // `shown` is derived from add-stack actions; a removal is not an
                // impression, so it doesn't count toward `shown`. A tally whose
                // impressions overflow u32 is dropped rather than sent wrong.
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
                    // The deck the swipes belong to. Lets the server derive the
                    // richer generalized-context per-otag signal, and (for
                    // non-Commander decks) is the sole context key.
                    deck_id: Some(deck_id),
                    shown,
                    added: t.added,
                    skipped: t.skipped,
                    maybed: t.maybed,
                    removed: t.removed,
                })
            })
            .collect();

        let select_signals: Vec<CommanderSelectDelta> = self
            .inner
            .select_signals
            .lock()
            .map(|mut map| std::mem::take(&mut *map))
            .unwrap_or_default()
            .into_iter()
            .filter_map(|(card, t)| {
                // Dropped rather than sent wrong, as with the signals above.
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

        let client_errors: Vec<ClientErrorReport> = self
            .inner
            .errors
            .lock()
            .map(|mut map| std::mem::take(&mut *map))
            .unwrap_or_default()
            .into_iter()
            .map(|(key, count)| ClientErrorReport {
                screen: key.screen.to_string(),
                component: key.component.to_string(),
                action: key.action.to_string(),
                kind: key.kind.to_string(),
                message: key.message,
                count,
                client_version: env!("CARGO_PKG_VERSION").to_string(),
                platform: ClientPlatform::CURRENT.as_str().to_string(),
            })
            .collect();

        let pile_ends: Vec<PileEndReport> = self
            .inner
            .pile_ends
            .lock()
            .map(|mut ends| std::mem::take(&mut *ends))
            .unwrap_or_default();

        if right == 0
            && left == 0
            && up == 0
            && down == 0
            && searches == 0
            && signals.is_empty()
            && select_signals.is_empty()
            && client_errors.is_empty()
            && pile_ends.is_empty()
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
            // batch field stays empty, kept for the wire format.
            deck_skips: Vec::new(),
            signals,
            select_signals,
            client_errors,
            pile_ends,
        })
    }
}

/// Report-kind slug for a status-carrying [`ClientError`] variant. The slugs
/// are stored server-side in `client_errors.kind`, so they are fixed strings.
fn api_error_kind(api: &ClientError) -> &'static str {
    match api {
        ClientError::Unauthorized(_) => "api_unauthorized",
        ClientError::Forbidden(_) => "api_forbidden",
        ClientError::NotFound(_) => "api_not_found",
        ClientError::UnprocessableEntity(_) => "api_unprocessable",
        ClientError::TooManyRequests(_) => "api_too_many_requests",
        ClientError::Conflict(_) => "api_conflict",
        ClientError::BadRequest(_) => "api_bad_request",
        ClientError::PayloadTooLarge(_) => "api_payload_too_large",
        // Network and Decode are handled before this is reached.
        ClientError::InternalServerError(_) | ClientError::Network(_) | ClientError::Decode(_) => {
            "api_internal"
        }
    }
}

/// Reduces a decode-error message to its shape: serde errors quote fragments
/// of the input around the failure point (`invalid type: string "..."`), and
/// response-body content must never ride a report or a log line. Double-quoted
/// spans collapse to `"…"`; field names in backticks and line/column positions
/// survive: they're the useful part.
fn sanitized_decode_message(detail: &str) -> String {
    let mut out = String::with_capacity(detail.len().min(128));
    let mut in_quotes = false;
    for c in detail.chars() {
        if c == '"' {
            if in_quotes {
                out.push('"');
            } else {
                out.push_str("\"…");
            }
            in_quotes = !in_quotes;
        } else if !in_quotes {
            out.push(c);
        }
    }
    if in_quotes {
        out.push('"');
    }
    out
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;
    use zwipe_client::ClientError;
    use zwipe_core::http::contracts::metrics::{HttpUsageBatch, PileEndKind, PileEndReport};

    use super::{CardTally, SelectTally, UsageBuffer};
    use crate::inbound::components::interactions::swipe::direction::Direction;

    #[test]
    fn snapshot_drains_once_and_resets() {
        let buffer = UsageBuffer::new();
        let deck = Uuid::new_v4();
        let card = Uuid::new_v4();

        buffer.record_swipe(Direction::Left);
        buffer.record_signal(deck, Some(card), Direction::Left);
        let batch = buffer.snapshot_and_zero().unwrap();
        assert_eq!(batch.swipes_left, 1);
        assert_eq!(batch.signals.len(), 1);
        let delta = batch.signals.first().unwrap();
        // The client pushes deck_id only; all context is derived server-side.
        assert_eq!(delta.deck_id, Some(deck));
        assert!(batch.deck_skips.is_empty());

        assert!(buffer.snapshot_and_zero().is_none());
    }

    #[test]
    fn a_full_tally_stays_full() {
        let buffer = UsageBuffer::new();
        let (deck, card) = (Uuid::new_v4(), Uuid::new_v4());
        let full = CardTally {
            added: u32::MAX,
            ..CardTally::default()
        };
        buffer
            .inner
            .signals
            .lock()
            .unwrap()
            .insert((card, deck), full);
        buffer.record_signal(deck, Some(card), Direction::Right);
        let tally = buffer.inner.signals.lock().unwrap()[&(card, deck)];
        assert_eq!(tally.added, u32::MAX);
    }

    #[test]
    fn a_shown_count_past_u32_drops_its_delta() {
        let buffer = UsageBuffer::new();
        let (deck, card) = (Uuid::new_v4(), Uuid::new_v4());
        let tally = CardTally {
            added: u32::MAX,
            skipped: 1,
            ..CardTally::default()
        };
        buffer
            .inner
            .signals
            .lock()
            .unwrap()
            .insert((card, deck), tally);
        let select = SelectTally {
            selected: u32::MAX,
            skipped: 1,
        };
        buffer
            .inner
            .select_signals
            .lock()
            .unwrap()
            .insert(card, select);
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
        let buffer = UsageBuffer::new();
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
    fn signal_carries_deck_context() {
        let buffer = UsageBuffer::new();
        let deck = Uuid::new_v4();
        let card = Uuid::new_v4();

        buffer.record_signal(deck, Some(card), Direction::Right);
        let batch = buffer.snapshot_and_zero().unwrap();
        assert_eq!(batch.signals.len(), 1);
        let delta = batch.signals.first().unwrap();
        assert_eq!(delta.deck_id, Some(deck));
        assert_eq!(delta.card_oracle_id, card);
        assert_eq!(delta.added, 1);
    }

    #[test]
    fn signal_with_no_card_oracle_id_is_dropped() {
        let buffer = UsageBuffer::new();
        let deck = Uuid::new_v4();
        buffer.record_signal(deck, None, Direction::Right);
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

    #[test]
    fn report_error_dedupes_and_caps() {
        let buffer = UsageBuffer::new();
        let error = ClientError::UnprocessableEntity("deck limit reached".to_string());

        // The same error three times → one entry, count 3.
        for _ in 0..3 {
            buffer.report_error("deck_edit", "", "save_profile", &error);
        }
        // Distinct actions beyond the cap are dropped, not accumulated.
        for i in 0..(HttpUsageBatch::MAX_CLIENT_ERRORS_PER_FLUSH + 10) {
            let action: &'static str = Box::leak(format!("action_{i}").into_boxed_str());
            buffer.report_error("deck_edit", "", action, &error);
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
        assert_ne!(repeated.client_version, "");

        // Drained: a second snapshot has nothing.
        assert!(buffer.snapshot_and_zero().is_none());
    }

    #[test]
    fn network_errors_are_never_buffered() {
        let buffer = UsageBuffer::new();
        buffer.report_error(
            "deck_edit",
            "",
            "save_profile",
            &ClientError::from(
                reqwest::Client::new()
                    .get("not a url")
                    .build()
                    .expect_err("a relative url fails to build"),
            ),
        );
        assert!(buffer.snapshot_and_zero().is_none());
    }

    #[test]
    fn decode_messages_lose_quoted_input_fragments() {
        let buffer = UsageBuffer::new();
        buffer.report_error(
            "deck_view",
            "",
            "load_deck",
            &ClientError::from(
                serde_json::from_str::<u32>("\"secret payload contents\"")
                    .expect_err("a string is not a u32"),
            ),
        );
        let batch = buffer.snapshot_and_zero().expect("decode error buffered");
        let report = batch.client_errors.first().unwrap();
        assert!(
            !report.message.contains("secret payload contents"),
            "quoted input stripped: {}",
            report.message
        );
        assert!(
            report.message.contains("expected u32 at line 1 column 25"),
            "error shape survives: {}",
            report.message
        );
    }
}
