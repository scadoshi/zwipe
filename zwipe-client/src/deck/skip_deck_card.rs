//! Post a single durable skip (and its undo) for a deck.

use crate::{ClientError, ZwipeClient};
use uuid::Uuid;
use zwipe_core::{
    domain::auth::models::session::Session,
    http::{
        contracts::deck::HttpSkipDeckCard,
        endpoints::deck::{SkipDeckCard, UnskipDeckCard},
    },
};

impl ZwipeClient {
    /// Posts a single deck-card skip.
    pub async fn skip_deck_card(
        &self,
        deck_id: Uuid,
        oracle_id: Uuid,
        session: &Session,
    ) -> Result<(), ClientError> {
        let body = serde_json::to_value(HttpSkipDeckCard { oracle_id })?;
        self.call(SkipDeckCard(deck_id, body), Some(session)).await
    }

    /// Undoes a single deck-card skip.
    pub async fn unskip_deck_card(
        &self,
        deck_id: Uuid,
        oracle_id: Uuid,
        session: &Session,
    ) -> Result<(), ClientError> {
        self.call(UnskipDeckCard(deck_id, oracle_id), Some(session))
            .await
    }
}
