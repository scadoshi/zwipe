//! Fetch all oracle text words.

use crate::outbound::client::{ClientError, ZwipeClient};
use zwipe_core::http::endpoints::card::GetOracleWords;

impl ZwipeClient {
    /// Fetches the list of all normalized oracle text words.
    pub async fn get_oracle_words(&self) -> Result<Vec<String>, ClientError> {
        self.call(GetOracleWords, None).await
    }
}
