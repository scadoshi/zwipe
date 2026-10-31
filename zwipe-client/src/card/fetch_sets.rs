//! Fetch all card sets.

use zwipe_core::http::endpoints::card::GetSets;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Fetches the list of all card sets.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn fetch_sets(&self) -> Result<Vec<String>, ClientError> {
        self.call(GetSets, None).await
    }
}
