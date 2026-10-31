//! Read a deck through its public share token.

use uuid::Uuid;
use zwipe_core::http::contracts::deck::HttpSharedDeck;
use zwipe_core::http::endpoints::deck::GetSharedDeck;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Fetches a shared deck by its public token.
    ///
    /// Unauthenticated: a share link is meant to open for anyone. A revoked or
    /// unknown token comes back as [`ClientError::NotFound`], which callers
    /// render as "not shared" rather than as an error.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn fetch_shared_deck(&self, token: Uuid) -> Result<HttpSharedDeck, ClientError> {
        self.call(GetSharedDeck(token), None).await
    }
}
