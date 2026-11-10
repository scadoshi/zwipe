//! Fetch all printings of a card by oracle ID.

use std::time::Duration;

use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::http::endpoints::card::GetPrintings;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Fetches all printings of a card by oracle ID, giving up after
    /// [`PRINTINGS_TIMEOUT`] so a stalled link ends in Retry rather than a
    /// skeleton that never resolves.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete or times out, the server
    /// answers with an error status, or the reply doesn't decode.
    pub async fn fetch_printings(&self, oracle_id: Uuid) -> Result<Vec<Card>, ClientError> {
        self.call_within(GetPrintings(oracle_id), None, PRINTINGS_TIMEOUT)
            .await
    }
}

/// How long the Printings sheet waits for its list before offering Retry:
/// long enough for a slow mobile link to bring a card with many printings,
/// short of the native client's 30-second limit for every call.
pub const PRINTINGS_TIMEOUT: Duration = Duration::from_secs(20);
