//! Fetch all unique artist names.

use zwipe_core::http::endpoints::card::GetArtists;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Fetches the list of all unique card artists.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn fetch_artists(&self) -> Result<Vec<String>, ClientError> {
        self.call(GetArtists, None).await
    }
}
