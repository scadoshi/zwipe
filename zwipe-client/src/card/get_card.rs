//! Fetch a single card by ID.

use crate::{ClientError, ZwipeClient};
use uuid::Uuid;
use zwipe_core::{domain::card::Card, http::endpoints::card::GetCard};

impl ZwipeClient {
    /// Fetches a single card by its Scryfall data ID.
    pub async fn get_card(&self, scryfall_data_id: Uuid) -> Result<Card, ClientError> {
        self.call(GetCard(scryfall_data_id), None).await
    }
}
