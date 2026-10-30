//! Fetch all printings of a card by oracle ID.

use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::http::endpoints::card::GetPrintings;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Fetches all printings of a card by oracle ID.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn get_printings(&self, oracle_id: Uuid) -> Result<Vec<Card>, ClientError> {
        self.call(GetPrintings(oracle_id), None).await
    }
}
