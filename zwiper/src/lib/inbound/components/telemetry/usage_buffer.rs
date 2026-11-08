//! The app's handle on zwipe-ui's shared usage buffer, taking the app's own
//! swipe [`Direction`](crate::inbound::components::interactions::swipe::direction::Direction)
//! and [`ClientError`](zwipe_client::ClientError).

use uuid::Uuid;
use zwipe_client::ClientError;
use zwipe_core::domain::auth::models::platform::ClientPlatform;
use zwipe_core::http::contracts::metrics::{HttpUsageBatch, PileEndKind};
use zwipe_ui::telemetry::{Failure, SharedUsageBuffer, SwipeDirection};

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

/// The report a failed request makes.
pub fn failure(error: &ClientError) -> Failure {
    match error {
        ClientError::Network(_) => Failure::Network(error.to_string()),
        ClientError::Decode(detail) => Failure::Decode(detail.to_string()),
        api => Failure::Status(api.status().unwrap_or(500), api.to_string()),
    }
}

/// The shared usage buffer. Cloning shares it.
#[derive(Debug, Clone)]
pub struct UsageBuffer {
    inner: SharedUsageBuffer,
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
            inner: SharedUsageBuffer::new(ClientPlatform::CURRENT, env!("CARGO_PKG_VERSION")),
        }
    }

    /// Records one swipe in the given direction.
    pub fn record_swipe(&self, direction: Direction) {
        self.inner.record_swipe(direction.into());
    }

    /// Records one card search.
    pub fn record_search(&self) {
        self.inner.record_search();
    }

    /// Records one add-stack swipe as a suggestion signal for
    /// `(card, deck)`.
    pub fn record_signal(&self, deck_id: Uuid, card_oracle_id: Option<Uuid>, direction: Direction) {
        self.inner
            .record_signal(deck_id, card_oracle_id, direction.into());
    }

    /// Records one swipe select decision.
    pub fn record_select_signal(&self, card_oracle_id: Option<Uuid>, direction: Direction) {
        self.inner
            .record_select_signal(card_oracle_id, direction.into());
    }

    /// Records one deliberate removal of a card from a deck.
    pub fn record_removal(&self, deck_id: Uuid, card_oracle_id: Option<Uuid>) {
        self.inner.record_removal(deck_id, card_oracle_id);
    }

    /// Records one time a deck's add-screen pile ran out.
    pub fn record_pile_end(&self, deck_id: Uuid, kind: PileEndKind, synergy: bool) {
        self.inner.record_pile_end(deck_id, kind, synergy);
    }

    /// Reports a handled error that was just shown to the user, and logs it.
    /// See [`SharedUsageBuffer::report_error`].
    pub fn report_error(
        &self,
        screen: &'static str,
        component: &'static str,
        action: &'static str,
        error: &ClientError,
    ) {
        self.inner
            .report_error(screen, component, action, &failure(error));
    }

    /// Drops everything buffered without sending it.
    pub fn clear(&self) {
        self.inner.clear();
    }

    /// Snapshots everything buffered into a batch and empties the buffer.
    ///
    /// Returns `None` when nothing was buffered.
    pub fn snapshot_and_zero(&self) -> Option<HttpUsageBatch> {
        self.inner.snapshot_and_zero()
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
