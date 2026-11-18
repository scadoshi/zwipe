//! Fetch the caller's owned printings of one card.

use uuid::Uuid;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::http::endpoints::collection::GetOwnedPrintings;

use crate::card::fetch_printings::PRINTINGS_TIMEOUT;
use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Fetches the signed-in user's owned printings of a card by oracle id, oldest release
    /// first, within [`PRINTINGS_TIMEOUT`] as the full printings list is.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete or times out, the server
    /// answers with an error status, or the reply doesn't decode.
    pub async fn fetch_owned_printings(
        &self,
        oracle_id: Uuid,
        session: &Session,
    ) -> Result<Vec<Card>, ClientError> {
        self.call_within(
            GetOwnedPrintings(oracle_id),
            Some(session),
            PRINTINGS_TIMEOUT,
        )
        .await
    }
}
