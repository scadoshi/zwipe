//! Fetch all keyword abilities.

use zwipe_core::http::endpoints::card::GetKeywords;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Fetches the list of all keyword abilities (flying, trample, etc.).
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn fetch_keywords(&self) -> Result<Vec<String>, ClientError> {
        self.call(GetKeywords, None).await
    }
}
