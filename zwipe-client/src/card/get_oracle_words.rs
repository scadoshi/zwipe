//! Fetch all oracle text words.

use zwipe_core::http::endpoints::card::GetOracleWords;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Fetches the list of all normalized oracle text words.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn fetch_oracle_words(&self) -> Result<Vec<String>, ClientError> {
        self.call(GetOracleWords, None).await
    }
}
