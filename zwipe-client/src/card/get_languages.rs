//! Fetch all available languages.

use zwipe_core::http::endpoints::card::GetLanguages;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Fetches the list of all available card languages.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn get_languages(&self) -> Result<Vec<String>, ClientError> {
        self.call(GetLanguages, None).await
    }
}
