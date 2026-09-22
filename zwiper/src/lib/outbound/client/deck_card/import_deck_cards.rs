//! Import cards into a deck from plain-text decklist.

use crate::outbound::client::{ClientError, ZwipeClient};
use uuid::Uuid;
use zwipe_core::{
    domain::{
        auth::models::session::Session,
        deck::{ImportMode, requests::import_deck_cards::ImportDeckCardsResult},
    },
    http::{contracts::deck_card::HttpImportDeckCards, endpoints::deck::ImportDeckCards},
};

impl ZwipeClient {
    /// Imports cards into a deck from plain text.
    pub async fn import_deck_cards(
        &self,
        deck_id: Uuid,
        text: &str,
        board: Option<&str>,
        mode: ImportMode,
        session: &Session,
    ) -> Result<ImportDeckCardsResult, ClientError> {
        let body = serde_json::to_value(HttpImportDeckCards {
            text: text.to_string(),
            board: board.map(|b| b.to_string()),
            mode,
        })?;
        self.call(ImportDeckCards(deck_id, body), Some(session))
            .await
    }
}
