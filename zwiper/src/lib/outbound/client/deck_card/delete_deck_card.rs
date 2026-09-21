//! Remove a card from a deck.

use crate::outbound::client::{ClientError, ZwipeClient};
use std::future::Future;
use uuid::Uuid;
use zwipe_core::{domain::auth::models::session::Session, http::endpoints::deck::DeleteDeckCard};

/// Trait for removing cards from a deck.
#[allow(missing_docs)]
pub trait ClientDeleteDeckCard {
    fn delete_deck_card(
        &self,
        deck_id: Uuid,
        scryfall_data_id: Uuid,
        session: &Session,
    ) -> impl Future<Output = Result<(), ClientError>> + Send;
}

impl ClientDeleteDeckCard for ZwipeClient {
    async fn delete_deck_card(
        &self,
        deck_id: Uuid,
        scryfall_data_id: Uuid,
        session: &Session,
    ) -> Result<(), ClientError> {
        self.call(DeleteDeckCard(deck_id, scryfall_data_id), Some(session))
            .await
    }
}
