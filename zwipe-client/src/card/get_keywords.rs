//! Fetch all keyword abilities.

use zwipe_core::http::endpoints::card::GetKeywords;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Fetches the list of all keyword abilities (flying, trample, etc.).
    pub async fn get_keywords(&self) -> Result<Vec<String>, ClientError> {
        self.call(GetKeywords, None).await
    }
}
