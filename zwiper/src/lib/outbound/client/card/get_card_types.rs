//! Fetch all card types.

use crate::outbound::client::{ClientError, ZwipeClient};
use zwipe_core::http::endpoints::card::GetCardTypes;

impl ZwipeClient {
    /// Fetches the list of all card types (creature, instant, etc.).
    pub async fn get_card_types(&self) -> Result<Vec<String>, ClientError> {
        self.call(GetCardTypes, None).await
    }
}
