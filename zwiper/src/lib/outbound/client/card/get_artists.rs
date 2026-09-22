//! Fetch all unique artist names.

use crate::outbound::client::{ClientError, ZwipeClient};
use zwipe_core::http::endpoints::card::GetArtists;

impl ZwipeClient {
    /// Fetches the list of all unique card artists.
    pub async fn get_artists(&self) -> Result<Vec<String>, ClientError> {
        self.call(GetArtists, None).await
    }
}
