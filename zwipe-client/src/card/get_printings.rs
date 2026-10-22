//! Fetch all printings of a card by oracle ID.

use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::http::endpoints::card::GetPrintings;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Fetches all printings of a card by oracle ID.
    pub async fn get_printings(&self, oracle_id: Uuid) -> Result<Vec<Card>, ClientError> {
        self.call(GetPrintings(oracle_id), None).await
    }
}
