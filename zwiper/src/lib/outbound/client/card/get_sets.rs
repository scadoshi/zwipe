//! Fetch all card sets.

use crate::outbound::client::{ClientError, ZwipeClient};
use zwipe_core::http::endpoints::card::GetSets;

impl ZwipeClient {
    /// Fetches the list of all card sets.
    pub async fn get_sets(&self) -> Result<Vec<String>, ClientError> {
        self.call(GetSets, None).await
    }
}
