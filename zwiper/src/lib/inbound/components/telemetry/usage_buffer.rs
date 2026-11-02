//! The app's handle on zwipe-ui's usage buffer: one lock shared by every
//! screen and the flush tasks, plus the mapping from a
//! [`ClientError`](zwipe_client::ClientError) to the
//! report it makes.

use std::sync::{Arc, Mutex, MutexGuard};

use uuid::Uuid;
use zwipe_client::ClientError;
use zwipe_core::domain::auth::models::platform::ClientPlatform;
use zwipe_core::http::contracts::metrics::{HttpUsageBatch, PileEndKind};
use zwipe_ui::telemetry::{self, ErrorKind, HandledError, SwipeDirection};

use crate::inbound::components::interactions::swipe::direction::Direction;

impl From<Direction> for SwipeDirection {
    fn from(direction: Direction) -> Self {
        match direction {
            Direction::Left => Self::Left,
            Direction::Right => Self::Right,
            Direction::Up => Self::Up,
            Direction::Down => Self::Down,
        }
    }
}

/// The shared usage buffer. Cloning shares it.
///
/// Every method takes the lock for one call and releases it before
/// returning, so no guard outlives a call or crosses an `.await`. A poisoned
/// lock is logged and the call does nothing: telemetry never fails a screen.
#[derive(Debug, Clone)]
pub struct UsageBuffer {
    inner: Arc<Mutex<telemetry::UsageBuffer>>,
}

impl Default for UsageBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl UsageBuffer {
    /// An empty buffer whose reports name this device's platform and this
    /// build's version.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(telemetry::UsageBuffer::new(
                ClientPlatform::CURRENT,
                env!("CARGO_PKG_VERSION"),
            ))),
        }
    }

    /// The buffer, or `None` with a warning when a panic poisoned its lock.
    fn lock(&self) -> Option<MutexGuard<'_, telemetry::UsageBuffer>> {
        self.inner
            .lock()
            .inspect_err(|_| tracing::warn!("usage buffer lock poisoned, usage dropped"))
            .ok()
    }

    /// Records one swipe in the given direction.
    pub fn record_swipe(&self, direction: Direction) {
        if let Some(mut buffer) = self.lock() {
            buffer.record_swipe(direction.into());
        }
    }

    /// Records one card search.
    pub fn record_search(&self) {
        if let Some(mut buffer) = self.lock() {
            buffer.record_search();
        }
    }

    /// Records one add-stack swipe as a suggestion signal for
    /// `(card, deck)`. See [`telemetry::UsageBuffer::record_signal`].
    pub fn record_signal(&self, deck_id: Uuid, card_oracle_id: Option<Uuid>, direction: Direction) {
        if let Some(mut buffer) = self.lock() {
            buffer.record_signal(deck_id, card_oracle_id, direction.into());
        }
    }

    /// Records one swipe select decision. See
    /// [`telemetry::UsageBuffer::record_select_signal`].
    pub fn record_select_signal(&self, card_oracle_id: Option<Uuid>, direction: Direction) {
        if let Some(mut buffer) = self.lock() {
            buffer.record_select_signal(card_oracle_id, direction.into());
        }
    }

    /// Records one deliberate removal of a card from a deck.
    pub fn record_removal(&self, deck_id: Uuid, card_oracle_id: Option<Uuid>) {
        if let Some(mut buffer) = self.lock() {
            buffer.record_removal(deck_id, card_oracle_id);
        }
    }

    /// Records one time a deck's add-screen pile ran out.
    pub fn record_pile_end(&self, deck_id: Uuid, kind: PileEndKind, synergy: bool) {
        if let Some(mut buffer) = self.lock() {
            buffer.record_pile_end(deck_id, kind, synergy);
        }
    }

    /// Reports a handled error that was just shown to the user, and logs it
    /// (one call, two sinks: client logs and the server table tell the same
    /// story).
    ///
    /// `screen`, `component` and `action` come from
    /// [`super::vocabulary`]. `component` is `""` when the screen itself
    /// showed the error.
    ///
    /// `Network` errors are logged but never buffered: they're mostly the
    /// user's connectivity, and couldn't be reported over the same dead
    /// connection anyway. A `Decode` message keeps only its shape; see
    /// [`HandledError::new`].
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
            ClientError::Decode(detail) => (ErrorKind::Decode, detail.to_string()),
            api => (api_error_kind(api), api.to_string()),
        };
        let handled = HandledError::new(screen, component, action, kind, &message);
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

    /// Snapshots everything buffered into a batch and empties the buffer.
    ///
    /// Returns `None` when nothing was buffered.
    pub fn snapshot_and_zero(&self) -> Option<HttpUsageBatch> {
        self.lock()?.snapshot_and_zero()
    }
}

/// The report kind for a status-carrying [`ClientError`].
fn api_error_kind(api: &ClientError) -> ErrorKind {
    match api {
        ClientError::Unauthorized(_) => ErrorKind::Unauthorized,
        ClientError::Forbidden(_) => ErrorKind::Forbidden,
        ClientError::NotFound(_) => ErrorKind::NotFound,
        ClientError::UnprocessableEntity(_) => ErrorKind::Unprocessable,
        ClientError::TooManyRequests(_) => ErrorKind::TooManyRequests,
        ClientError::Conflict(_) => ErrorKind::Conflict,
        ClientError::BadRequest(_) => ErrorKind::BadRequest,
        ClientError::PayloadTooLarge(_) => ErrorKind::PayloadTooLarge,
        // Network and Decode are handled before this is reached.
        ClientError::InternalServerError(_) | ClientError::Network(_) | ClientError::Decode(_) => {
            ErrorKind::Internal
        }
    }
}

#[cfg(test)]
mod tests {
    use zwipe_client::ClientError;

    use super::{Direction, UsageBuffer};

    #[test]
    fn every_status_maps_to_its_stored_kind() {
        let cases = [
            (ClientError::Unauthorized(String::new()), "api_unauthorized"),
            (ClientError::Forbidden(String::new()), "api_forbidden"),
            (ClientError::NotFound(String::new()), "api_not_found"),
            (
                ClientError::UnprocessableEntity(String::new()),
                "api_unprocessable",
            ),
            (
                ClientError::TooManyRequests(String::new()),
                "api_too_many_requests",
            ),
            (ClientError::Conflict(String::new()), "api_conflict"),
            (ClientError::BadRequest(String::new()), "api_bad_request"),
            (
                ClientError::PayloadTooLarge(String::new()),
                "api_payload_too_large",
            ),
            (
                ClientError::InternalServerError(String::new()),
                "api_internal",
            ),
        ];
        for (error, slug) in cases {
            let buffer = UsageBuffer::new();
            buffer.report_error("deck_edit", "", "save_profile", &error);
            let batch = buffer.snapshot_and_zero().expect("error buffered");
            let report = batch.client_errors.first().expect("one report");
            assert_eq!(report.kind, slug, "{error:?}");
        }
    }

    #[test]
    fn report_error_dedupes_and_tags_this_build() {
        let buffer = UsageBuffer::new();
        let error = ClientError::UnprocessableEntity("deck limit reached".to_string());
        for _ in 0..3 {
            buffer.report_error("deck_edit", "", "save_profile", &error);
        }
        let batch = buffer.snapshot_and_zero().expect("errors buffered");
        let report = batch.client_errors.first().expect("one report");
        assert_eq!(report.count, 3);
        assert_eq!(report.message, "deck limit reached");
        assert_eq!(report.client_version, env!("CARGO_PKG_VERSION"));
        assert_eq!(
            report.platform,
            zwipe_core::domain::auth::models::platform::ClientPlatform::CURRENT.as_str()
        );
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
        let report = batch.client_errors.first().expect("one report");
        assert_eq!(report.kind, "decode");
        assert!(
            !report.message.contains("secret payload contents"),
            "quoted input stripped: {}",
            report.message
        );
    }

    #[test]
    fn clones_share_one_buffer() {
        let buffer = UsageBuffer::new();
        let other = buffer.clone();
        other.record_swipe(Direction::Up);
        other.record_signal(
            uuid::Uuid::new_v4(),
            Some(uuid::Uuid::new_v4()),
            Direction::Up,
        );
        let batch = buffer.snapshot_and_zero().expect("swipe buffered");
        assert_eq!(batch.swipes_up, 1);
        assert_eq!(batch.signals.first().map(|s| s.maybed), Some(1));
        assert!(other.snapshot_and_zero().is_none());
    }
}
