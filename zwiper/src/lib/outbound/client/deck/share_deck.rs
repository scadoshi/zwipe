//! Share / unshare a deck (public link token management).

use crate::outbound::client::{ClientError, ZwipeClient};
use std::future::Future;
use uuid::Uuid;
use zwipe_core::{
    domain::auth::models::session::Session,
    http::{
        contracts::deck::HttpDeckShareToken,
        endpoints::deck::{ShareDeck, UnshareDeck},
    },
};

/// Trait for creating and revoking a deck's public share link.
#[allow(missing_docs)]
pub trait ClientShareDeck {
    /// Generates (or regenerates) the share token. Re-sharing rotates it, so
    /// any previously shared link dies.
    fn share_deck(
        &self,
        deck_id: Uuid,
        session: &Session,
    ) -> impl Future<Output = Result<HttpDeckShareToken, ClientError>> + Send;

    /// Revokes the deck's share token (stops sharing).
    fn unshare_deck(
        &self,
        deck_id: Uuid,
        session: &Session,
    ) -> impl Future<Output = Result<(), ClientError>> + Send;
}

impl ClientShareDeck for ZwipeClient {
    async fn share_deck(
        &self,
        deck_id: Uuid,
        session: &Session,
    ) -> Result<HttpDeckShareToken, ClientError> {
        self.call(ShareDeck(deck_id), Some(session)).await
    }

    async fn unshare_deck(&self, deck_id: Uuid, session: &Session) -> Result<(), ClientError> {
        self.call(UnshareDeck(deck_id), Some(session)).await
    }
}
