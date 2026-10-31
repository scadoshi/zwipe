//! Fetch all card types.

use zwipe_core::http::endpoints::card::GetCardTypes;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Fetches the list of all card types (creature, instant, etc.).
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn fetch_card_types(&self) -> Result<Vec<String>, ClientError> {
        self.call(GetCardTypes, None).await
    }
}
