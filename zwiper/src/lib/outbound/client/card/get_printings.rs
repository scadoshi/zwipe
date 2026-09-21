//! Fetch all printings of a card by oracle ID.

use crate::outbound::client::{ClientError, ZwipeClient};
use std::future::Future;
use uuid::Uuid;
use zwipe_core::{domain::card::Card, http::endpoints::card::GetPrintings};

/// Trait for fetching all printings of a card by oracle ID.
#[allow(missing_docs)]
pub trait ClientGetPrintings {
    fn get_printings(
        &self,
        oracle_id: Uuid,
    ) -> impl Future<Output = Result<Vec<Card>, ClientError>> + Send;
}

impl ClientGetPrintings for ZwipeClient {
    async fn get_printings(&self, oracle_id: Uuid) -> Result<Vec<Card>, ClientError> {
        self.call(GetPrintings(oracle_id), None).await
    }
}
