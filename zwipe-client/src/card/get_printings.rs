//! Fetch all printings of a card by oracle ID.

use crate::{ClientError, ZwipeClient};
use uuid::Uuid;
use zwipe_core::{domain::card::Card, http::endpoints::card::GetPrintings};

impl ZwipeClient {
    /// Fetches all printings of a card by oracle ID.
    pub async fn get_printings(&self, oracle_id: Uuid) -> Result<Vec<Card>, ClientError> {
        self.call(GetPrintings(oracle_id), None).await
    }
}
