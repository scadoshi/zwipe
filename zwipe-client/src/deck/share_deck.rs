//! Share / unshare a deck (public link token management).

use uuid::Uuid;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::deck::HttpDeckShareToken;
use zwipe_core::http::endpoints::deck::{ShareDeck, UnshareDeck};

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Creates a deck's public share link.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn share_deck(
        &self,
        deck_id: Uuid,
        session: &Session,
    ) -> Result<HttpDeckShareToken, ClientError> {
        self.call(ShareDeck(deck_id), Some(session)).await
    }

    /// Revokes a deck's public share link.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn unshare_deck(&self, deck_id: Uuid, session: &Session) -> Result<(), ClientError> {
        self.call(UnshareDeck(deck_id), Some(session)).await
    }
}
