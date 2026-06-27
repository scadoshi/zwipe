//! Import an Archidekt deck's cards into an existing deck.

use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use tracing::info;
use uuid::Uuid;
use zwipe::inbound::http::{ApiError, routes::import_archidekt_deck_route};
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::deck::ImportMode;
use zwipe_core::domain::deck::requests::import_deck_cards::ImportDeckCardsResult;
use zwipe_core::http::contracts::deck::HttpImportArchidektDeck;

/// Trait for importing an Archidekt deck's cards into an existing deck.
///
/// The server fetches and parses the deck, resolves printings by Scryfall id,
/// and imports the cards exactly like the plain-text importer — same boards,
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
    ) -> impl Future<Output = Result<ImportDeckCardsResult, ApiError>> + Send;
}

impl ClientImportArchidektDeck for ZwipeClient {
    async fn import_archidekt_deck(
        &self,
        deck_id: Uuid,
        url: &str,
        board: Option<&str>,
        mode: ImportMode,
        session: &Session,
    ) -> Result<ImportDeckCardsResult, ApiError> {
        let mut request_url = self.app_config.backend_url.clone();
        request_url.set_path(&import_archidekt_deck_route(deck_id));

        let body = HttpImportArchidektDeck {
            url: url.to_string(),
            board: board.map(|b| b.to_string()),
            mode,
        };
        info!("POST {} body: {:?}", request_url, body);

        let response = self
            .client
            .post(request_url)
            .json(&body)
            .bearer_auth(&*session.access_token.value)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let result: ImportDeckCardsResult = response.json().await?;
                Ok(result)
            }
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
