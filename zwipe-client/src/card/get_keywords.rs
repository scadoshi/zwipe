//! Fetch all keyword abilities.

use crate::{ClientError, ZwipeClient};
use zwipe_core::http::endpoints::card::GetKeywords;

impl ZwipeClient {
    /// Fetches the list of all keyword abilities (flying, trample, etc.).
    pub async fn get_keywords(&self) -> Result<Vec<String>, ClientError> {
        self.call(GetKeywords, None).await
    }
}
