//! Fetch a single card by ID.

use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::http::endpoints::card::GetCard;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Fetches a single card by its Scryfall data ID.
    pub async fn get_card(&self, scryfall_data_id: Uuid) -> Result<Card, ClientError> {
        self.call(GetCard(scryfall_data_id), None).await
    }
}
