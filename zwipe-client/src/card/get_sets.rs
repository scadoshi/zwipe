//! Fetch all card sets.

use zwipe_core::http::endpoints::card::GetSets;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Fetches the list of all card sets.
    pub async fn get_sets(&self) -> Result<Vec<String>, ClientError> {
        self.call(GetSets, None).await
    }
}
