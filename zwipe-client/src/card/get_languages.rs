//! Fetch all available languages.

use zwipe_core::http::endpoints::card::GetLanguages;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Fetches the list of all available card languages.
    pub async fn get_languages(&self) -> Result<Vec<String>, ClientError> {
        self.call(GetLanguages, None).await
    }
}
