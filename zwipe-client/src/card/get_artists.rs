//! Fetch all unique artist names.

use zwipe_core::http::endpoints::card::GetArtists;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Fetches the list of all unique card artists.
    pub async fn get_artists(&self) -> Result<Vec<String>, ClientError> {
        self.call(GetArtists, None).await
    }
}
