//! Post a single durable skip (and its undo) for a deck.

use uuid::Uuid;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::deck::HttpSkipDeckCard;
use zwipe_core::http::endpoints::deck::{SkipDeckCard, UnskipDeckCard};

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Posts a single deck-card skip.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn skip_deck_card(
        &self,
        deck_id: Uuid,
        oracle_id: Uuid,
        session: &Session,
    ) -> Result<(), ClientError> {
        self.call(
            SkipDeckCard(deck_id, HttpSkipDeckCard { oracle_id }),
            Some(session),
        )
        .await
    }

    /// Undoes a single deck-card skip.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
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
