//! The usage buffer every screen of a client shares: one lock around [`UsageBuffer`], and the step from a failed request to the report it makes.

use std::sync::{Arc, Mutex, MutexGuard};

use uuid::Uuid;
use zwipe_core::domain::auth::models::platform::ClientPlatform;
use zwipe_core::http::contracts::metrics::{HttpUsageBatch, PileEndKind};

use super::{ErrorKind, HandledError, SwipeDirection, UsageBuffer};

/// How a request the user saw fail went wrong, as a client hands it to [`SharedUsageBuffer::report_error`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Failure {
    /// The request never got an answer. Logged, never buffered: it is mostly the user's connection, and could not be reported over it anyway.
    Network(String),
    /// The answer arrived but did not parse. Only the message's shape is kept.
    Decode(String),
    /// The server answered with this status and message.
    Status(u16, String),
}

impl ErrorKind {
    /// The kind for an HTTP status; a status no kind names is [`ErrorKind::Internal`].
    #[must_use]
    pub const fn from_status(status: u16) -> Self {
        match status {
            400 => Self::BadRequest,
            401 => Self::Unauthorized,
            403 => Self::Forbidden,
            404 => Self::NotFound,
            409 => Self::Conflict,
            413 => Self::PayloadTooLarge,
            422 => Self::Unprocessable,
            429 => Self::TooManyRequests,
            _ => Self::Internal,
        }
    }
}

/// The usage buffer behind one lock. Cloning shares it.
///
/// Every method takes the lock for one call and lets it go before returning, so no guard outlives a call or crosses an `.await`. A poisoned lock is logged and the call does nothing: telemetry never fails a screen.
#[derive(Debug, Clone)]
pub struct SharedUsageBuffer {
    inner: Arc<Mutex<UsageBuffer>>,
}

impl SharedUsageBuffer {
    /// An empty buffer whose error reports name `platform` and `client_version`.
    #[must_use]
    pub fn new(platform: ClientPlatform, client_version: &'static str) -> Self {
        Self {
            inner: Arc::new(Mutex::new(UsageBuffer::new(platform, client_version))),
        }
    }

    /// The buffer, or `None` with a warning when a panic poisoned its lock.
    fn lock(&self) -> Option<MutexGuard<'_, UsageBuffer>> {
        self.inner
            .lock()
            .inspect_err(|_| tracing::warn!("usage buffer lock poisoned, usage dropped"))
            .ok()
    }

    /// Records one committed swipe in `direction`.
    pub fn record_swipe(&self, direction: SwipeDirection) {
        if let Some(mut buffer) = self.lock() {
            buffer.record_swipe(direction);
        }
    }

    /// Records one card search.
    pub fn record_search(&self) {
        if let Some(mut buffer) = self.lock() {
            buffer.record_search();
        }
    }

    /// Records one Add swipe as a suggestion signal for `(card, deck)`. See [`UsageBuffer::record_signal`].
    pub fn record_signal(
        &self,
        deck_id: Uuid,
        card_oracle_id: Option<Uuid>,
        direction: SwipeDirection,
    ) {
        if let Some(mut buffer) = self.lock() {
            buffer.record_signal(deck_id, card_oracle_id, direction);
        }
    }

    /// Records one Swipe select decision. See [`UsageBuffer::record_select_signal`].
    pub fn record_select_signal(&self, card_oracle_id: Option<Uuid>, direction: SwipeDirection) {
        if let Some(mut buffer) = self.lock() {
            buffer.record_select_signal(card_oracle_id, direction);
        }
    }

    /// Records one deliberate removal of a card from a deck.
    pub fn record_removal(&self, deck_id: Uuid, card_oracle_id: Option<Uuid>) {
        if let Some(mut buffer) = self.lock() {
            buffer.record_removal(deck_id, card_oracle_id);
        }
    }

    /// Records one time a deck's Add pile ran out.
    pub fn record_pile_end(&self, deck_id: Uuid, kind: PileEndKind, synergy: bool) {
        if let Some(mut buffer) = self.lock() {
            buffer.record_pile_end(deck_id, kind, synergy);
        }
    }

    /// Reports a failure the user was just shown, and logs it, so the client log and the server table tell one story. `screen`, `component` and `action` come from [`vocabulary`](super::vocabulary); `component` is [`component::NONE`](super::vocabulary::component::NONE) when the screen itself showed it. A [`Failure::Network`] is logged only.
    pub fn report_error(
        &self,
        screen: &'static str,
        component: &'static str,
        action: &'static str,
        failure: &Failure,
    ) {
        let (kind, message) = match failure {
            Failure::Network(message) => {
                tracing::warn!(
                    screen,
                    component,
                    action,
                    kind = "network",
                    "error shown: {message}"
                );
                return;
            }
            Failure::Decode(message) => (ErrorKind::Decode, message.as_str()),
            Failure::Status(status, message) => (ErrorKind::from_status(*status), message.as_str()),
        };
        let handled = HandledError::new(screen, component, action, kind, message);
        tracing::warn!(
            screen,
            component,
            action,
            kind = kind.as_str(),
            "error shown: {}",
            handled.message()
        );
        if let Some(mut buffer) = self.lock() {
            buffer.report_error(handled);
        }
    }

    /// Drops everything buffered without sending it.
    pub fn clear(&self) {
        if let Some(mut buffer) = self.lock() {
            buffer.clear();
        }
    }

    /// Drains everything buffered into one batch, `None` when nothing was.
    #[must_use]
    pub fn snapshot_and_zero(&self) -> Option<HttpUsageBatch> {
        self.lock()?.snapshot_and_zero()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn buffer() -> SharedUsageBuffer {
        SharedUsageBuffer::new(ClientPlatform::Web, "9.9.9")
    }

    #[test]
    fn every_status_maps_to_its_stored_kind() {
        let cases = [
            (400, "api_bad_request"),
            (401, "api_unauthorized"),
            (403, "api_forbidden"),
            (404, "api_not_found"),
            (409, "api_conflict"),
            (413, "api_payload_too_large"),
            (422, "api_unprocessable"),
            (429, "api_too_many_requests"),
            (500, "api_internal"),
            (502, "api_internal"),
        ];
        for (status, slug) in cases {
            let buffer = buffer();
            buffer.report_error(
                "deck_edit",
                "",
                "save_profile",
                &Failure::Status(status, String::new()),
            );
            let batch = buffer.snapshot_and_zero().unwrap();
            assert_eq!(
                batch.client_errors.first().map(|r| r.kind.as_str()),
                Some(slug),
                "{status}"
            );
        }
    }

    #[test]
    fn a_report_is_counted_and_tagged_with_the_platform_and_version() {
        let buffer = buffer();
        let failure = Failure::Status(422, "deck limit reached".to_owned());
        for _ in 0..3 {
            buffer.report_error("deck_edit", "", "save_profile", &failure);
        }
        let batch = buffer.snapshot_and_zero().unwrap();
        let report = batch.client_errors.first().unwrap();
        assert_eq!(report.count, 3);
        assert_eq!(report.platform, "web");
        assert_eq!(report.client_version, "9.9.9");
        assert!(buffer.snapshot_and_zero().is_none());
    }

    #[test]
    fn network_failures_are_never_buffered() {
        let buffer = buffer();
        buffer.report_error(
            "deck_edit",
            "",
            "save_profile",
            &Failure::Network("offline".to_owned()),
        );
        assert!(buffer.snapshot_and_zero().is_none());
    }

    #[test]
    fn decode_messages_lose_quoted_input() {
        let buffer = buffer();
        let failure =
            Failure::Decode("invalid type: string \"secret payload\", expected u32".to_owned());
        buffer.report_error("deck_view", "", "load_deck", &failure);
        let batch = buffer.snapshot_and_zero().unwrap();
        let report = batch.client_errors.first().unwrap();
        assert_eq!(report.kind, "decode");
        assert!(
            !report.message.contains("secret payload"),
            "{}",
            report.message
        );
    }

    #[test]
    fn clones_share_one_buffer() {
        let buffer = buffer();
        let other = buffer.clone();
        other.record_swipe(SwipeDirection::Up);
        let batch = buffer.snapshot_and_zero().unwrap();
        assert_eq!(batch.swipes_up, 1);
        assert!(other.snapshot_and_zero().is_none());
    }
}
