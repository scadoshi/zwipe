//! Fetch all available languages.

use crate::{ClientError, ZwipeClient};
use zwipe_core::http::endpoints::card::GetLanguages;

impl ZwipeClient {
    /// Fetches the list of all available card languages.
    pub async fn get_languages(&self) -> Result<Vec<String>, ClientError> {
        self.call(GetLanguages, None).await
    }
}
