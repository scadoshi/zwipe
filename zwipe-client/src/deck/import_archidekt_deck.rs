//! Import an Archidekt deck's cards into an existing deck.

use uuid::Uuid;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::deck::ImportMode;
use zwipe_core::domain::deck::requests::import_deck_cards::ImportDeckCardsResult;
use zwipe_core::http::contracts::deck::HttpImportArchidektDeck;
use zwipe_core::http::endpoints::deck::ImportArchidektDeck;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Imports an Archidekt deck's cards into an existing deck.
    ///
    /// The server fetches and parses the deck, resolves printings by Scryfall id,
    /// and imports the cards exactly like the plain-text importer: same boards,
    /// same add/replace modes, same result shape.
    pub async fn import_archidekt_deck(
        &self,
        deck_id: Uuid,
        url: &str,
        board: Option<&str>,
        mode: ImportMode,
        session: &Session,
    ) -> Result<ImportDeckCardsResult, ClientError> {
        self.call(
            ImportArchidektDeck(
                deck_id,
                HttpImportArchidektDeck {
                    url: url.to_string(),
                    board: board.map(|b| b.to_string()),
                    mode,
                },
            ),
            Some(session),
        )
        .await
    }
}
