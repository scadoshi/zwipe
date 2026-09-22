//! Read a deck through its public share token.

use crate::{ClientError, ZwipeClient};
use uuid::Uuid;
use zwipe_core::http::{contracts::deck::HttpSharedDeck, endpoints::deck::GetSharedDeck};

impl ZwipeClient {
    /// Fetches a shared deck by its public token.
    ///
    /// Unauthenticated: a share link is meant to open for anyone. A revoked or
    /// unknown token comes back as [`ClientError::NotFound`], which callers
    /// render as "not shared" rather than as an error.
    pub async fn get_shared_deck(&self, token: Uuid) -> Result<HttpSharedDeck, ClientError> {
        self.call(GetSharedDeck(token), None).await
    }
}
