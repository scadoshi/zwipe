//! Fetch a single card by ID.

use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::http::endpoints::card::GetCard;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Fetches a single card by its Scryfall data ID.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn get_card(&self, scryfall_data_id: Uuid) -> Result<Card, ClientError> {
        self.call(GetCard(scryfall_data_id), None).await
    }
}
