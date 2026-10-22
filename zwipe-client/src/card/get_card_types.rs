//! Fetch all card types.

use zwipe_core::http::endpoints::card::GetCardTypes;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Fetches the list of all card types (creature, instant, etc.).
    pub async fn get_card_types(&self) -> Result<Vec<String>, ClientError> {
        self.call(GetCardTypes, None).await
    }
}
