//! In-memory swipe/search counters with snapshot-and-zero semantics.

use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU32, Ordering},
    },
};

use uuid::Uuid;
use zwipe::inbound::http::ApiError;
use zwipe_core::{
    domain::auth::models::platform::ClientPlatform,
    http::contracts::metrics::{
        CardSignalDelta, ClientErrorReport, CommanderSelectDelta, HttpUsageBatch,
    },
};

use crate::{
    inbound::components::interactions::swipe::direction::Direction, outbound::client::ClientError,
};

/// Signal buffer key: `(card oracle id, deck id)`. The deck id rides the wire
/// as the sole context key — the server derives the commander (EDH) or the
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
    /// flush window — an error loop increments existing counts but never
    /// grows the map past the cap.
    errors: Mutex<HashMap<ErrorKey, u32>>,
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
    /// `Down` (undo) is ignored — only added/skipped/maybed express intent. No-op
    /// if the card has no oracle id.
    pub fn record_signal(&self, deck_id: Uuid, card_oracle_id: Option<Uuid>, direction: Direction) {
        let Some(card) = card_oracle_id else {
            return;
        };
        let Ok(mut map) = self.inner.signals.lock() else {
            return;
        };
        let tally = map.entry((card, deck_id)).or_default();
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

    /// Reports a handled error that was just surfaced to the user, and mirrors
    /// it into the local tracing log (one call, two sinks — client logs and
    /// the server table tell the same story).
    ///
    /// `screen`/`component`/`action` come from [`super::vocabulary`] — the
    /// closed, module-path-derived breadcrumb. `component` is `""` when the
    /// screen itself surfaced the error.
    ///
    /// `Network` errors are logged but never buffered: they're mostly the
    /// user's connectivity, and couldn't be reported over the same dead
    /// connection anyway. `Decode` messages are reduced to the error's shape
    /// (quoted input fragments stripped) — response bodies must never ride a
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
            ClientError::Decode(detail) => ("decode", sanitized_decode_message(detail)),
            ClientError::Api(api) => (api_error_kind(api), api.to_string()),
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

    /// Records one deliberate removal of a card from a deck — a delayed negative
    /// signal, distinct from an add-stack skip. Keyed by `(card, deck)`. No-op
    /// if the card has no oracle id.
    pub fn record_removal(&self, deck_id: Uuid, card_oracle_id: Option<Uuid>) {
        let Some(card) = card_oracle_id else {
            return;
        };
        if let Ok(mut map) = self.inner.signals.lock() {
            map.entry((card, deck_id)).or_default().removed += 1;
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
            .map(|((card, deck_id), t)| CardSignalDelta {
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

        if right == 0
            && left == 0
            && up == 0
            && down == 0
            && searches == 0
            && signals.is_empty()
            && select_signals.is_empty()
            && client_errors.is_empty()
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
            client_errors,
        })
    }
}

/// Report-kind slug for an [`ApiError`] variant.
fn api_error_kind(api: &ApiError) -> &'static str {
    match api {
        ApiError::Unauthorized(_) => "api_unauthorized",
        ApiError::Forbidden(_) => "api_forbidden",
        ApiError::NotFound(_) => "api_not_found",
        ApiError::UnprocessableEntity(_) => "api_unprocessable",
        ApiError::TooManyRequests(_) => "api_too_many_requests",
        ApiError::InternalServerError(_) => "api_internal",
    }
}

/// Reduces a decode-error message to its shape: serde errors quote fragments
/// of the input around the failure point (`invalid type: string "..."`), and
/// response-body content must never ride a report or a log line. Double-quoted
/// spans collapse to `"…"`; field names in backticks and line/column positions
/// survive — they're the useful part.
fn sanitized_decode_message(detail: &str) -> String {
    let mut out = String::with_capacity(detail.len().min(128));
    let mut in_quotes = false;
    for c in detail.chars() {
        if c == '"' {
            if !in_quotes {
                out.push_str("\"…");
            } else {
                out.push('"');
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
#[allow(clippy::unwrap_used)]
mod tests {
    use super::UsageBuffer;
    use crate::{
        inbound::components::interactions::swipe::direction::Direction,
        outbound::client::ClientError,
    };
    use uuid::Uuid;
    use zwipe::inbound::http::ApiError;
    use zwipe_core::http::contracts::metrics::HttpUsageBatch;

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
        let error = ClientError::Api(ApiError::UnprocessableEntity(
            "deck limit reached".to_string(),
        ));

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
        assert!(!repeated.client_version.is_empty());

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
            &ClientError::Network("dns error: no such host".to_string()),
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
            &ClientError::Decode(
                "invalid type: string \"secret payload contents\", expected u32 at line 1 column 9"
                    .to_string(),
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
            report.message.contains("expected u32 at line 1 column 9"),
            "error shape survives: {}",
            report.message
        );
    }
}
