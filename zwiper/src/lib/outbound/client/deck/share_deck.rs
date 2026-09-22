//! Share / unshare a deck (public link token management).

use crate::outbound::client::{ClientError, ZwipeClient};
use uuid::Uuid;
use zwipe_core::{
    domain::auth::models::session::Session,
    http::{
        contracts::deck::HttpDeckShareToken,
        endpoints::deck::{ShareDeck, UnshareDeck},
    },
};

impl ZwipeClient {
    /// Creates and revoking a deck's public share link.
    pub async fn share_deck(
        &self,
        deck_id: Uuid,
        session: &Session,
    ) -> Result<HttpDeckShareToken, ClientError> {
        self.call(ShareDeck(deck_id), Some(session)).await
    }

    /// Creates and revoking a deck's public share link.

    pub async fn unshare_deck(&self, deck_id: Uuid, session: &Session) -> Result<(), ClientError> {
        self.call(UnshareDeck(deck_id), Some(session)).await
    }
}
