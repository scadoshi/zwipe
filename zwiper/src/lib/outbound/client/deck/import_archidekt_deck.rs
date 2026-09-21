//! Import an Archidekt deck's cards into an existing deck.

use crate::outbound::client::{ClientError, ZwipeClient};
use std::future::Future;
use uuid::Uuid;
use zwipe_core::{
    domain::{
        auth::models::session::Session,
        deck::{ImportMode, requests::import_deck_cards::ImportDeckCardsResult},
    },
    http::{contracts::deck::HttpImportArchidektDeck, endpoints::deck::ImportArchidektDeck},
};

/// Trait for importing an Archidekt deck's cards into an existing deck.
///
/// The server fetches and parses the deck, resolves printings by Scryfall id,
/// and imports the cards exactly like the plain-text importer: same boards,
/// same add/replace modes, same result shape.
#[allow(missing_docs)]
pub trait ClientImportArchidektDeck {
    fn import_archidekt_deck(
        &self,
        deck_id: Uuid,
        url: &str,
        board: Option<&str>,
        mode: ImportMode,
        session: &Session,
    ) -> impl Future<Output = Result<ImportDeckCardsResult, ClientError>> + Send;
}

impl ClientImportArchidektDeck for ZwipeClient {
    async fn import_archidekt_deck(
        &self,
        deck_id: Uuid,
        url: &str,
        board: Option<&str>,
        mode: ImportMode,
        session: &Session,
    ) -> Result<ImportDeckCardsResult, ClientError> {
        let body = serde_json::to_value(HttpImportArchidektDeck {
            url: url.to_string(),
            board: board.map(|b| b.to_string()),
            mode,
        })?;
        self.call(ImportArchidektDeck(deck_id, body), Some(session))
            .await
    }
}
