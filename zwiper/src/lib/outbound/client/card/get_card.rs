//! Fetch a single card by ID.

use crate::outbound::client::{ClientError, ZwipeClient};
use std::future::Future;
use uuid::Uuid;
use zwipe_core::{domain::card::Card, http::endpoints::card::GetCard};

/// Trait for fetching a single card by its Scryfall data ID.
#[allow(missing_docs)]
pub trait ClientGetCard {
    fn get_card(
        &self,
        scryfall_data_id: Uuid,
    ) -> impl Future<Output = Result<Card, ClientError>> + Send;
}

impl ClientGetCard for ZwipeClient {
    async fn get_card(&self, scryfall_data_id: Uuid) -> Result<Card, ClientError> {
        self.call(GetCard(scryfall_data_id), None).await
    }
}
