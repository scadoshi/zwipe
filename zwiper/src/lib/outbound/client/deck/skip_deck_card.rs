//! Post a single durable skip (and its undo) for a deck.

use crate::outbound::client::{ClientError, ZwipeClient};
use std::future::Future;
use uuid::Uuid;
use zwipe_core::{
    domain::auth::models::session::Session,
    http::{
        contracts::deck::HttpSkipDeckCard,
        endpoints::deck::{SkipDeckCard, UnskipDeckCard},
    },
};

/// Trait for posting and undoing a single deck-card skip.
#[allow(missing_docs)]
pub trait ClientSkipDeckCard {
    fn skip_deck_card(
        &self,
        deck_id: Uuid,
        oracle_id: Uuid,
        session: &Session,
    ) -> impl Future<Output = Result<(), ClientError>> + Send;

    fn unskip_deck_card(
        &self,
        deck_id: Uuid,
        oracle_id: Uuid,
        session: &Session,
    ) -> impl Future<Output = Result<(), ClientError>> + Send;
}

impl ClientSkipDeckCard for ZwipeClient {
    async fn skip_deck_card(
        &self,
        deck_id: Uuid,
        oracle_id: Uuid,
        session: &Session,
    ) -> Result<(), ClientError> {
        let body = serde_json::to_value(HttpSkipDeckCard { oracle_id })?;
        self.call(SkipDeckCard(deck_id, body), Some(session)).await
    }

    async fn unskip_deck_card(
        &self,
        deck_id: Uuid,
        oracle_id: Uuid,
        session: &Session,
    ) -> Result<(), ClientError> {
        self.call(UnskipDeckCard(deck_id, oracle_id), Some(session))
            .await
    }
}
