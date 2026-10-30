//! Import cards into a deck from plain-text decklist.

use uuid::Uuid;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::deck::ImportMode;
use zwipe_core::domain::deck::requests::import_deck_cards::ImportDeckCardsResult;
use zwipe_core::http::contracts::deck_card::HttpImportDeckCards;
use zwipe_core::http::endpoints::deck::ImportDeckCards;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Imports cards into a deck from plain text.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn import_deck_cards(
        &self,
        deck_id: Uuid,
        text: &str,
        board: Option<&str>,
        mode: ImportMode,
        session: &Session,
    ) -> Result<ImportDeckCardsResult, ClientError> {
        self.call(
            ImportDeckCards(
                deck_id,
                HttpImportDeckCards {
                    text: text.to_string(),
                    board: board.map(|b| b.to_string()),
                    mode,
                },
            ),
            Some(session),
        )
        .await
    }
}
