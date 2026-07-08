//! Import cards into a deck from plain-text decklist.

use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use tracing::info;
use uuid::Uuid;
use zwipe::inbound::http::{ApiError, routes::import_deck_cards_route};
use zwipe_core::{
    domain::{
        auth::models::session::Session,
        deck::{ImportMode, requests::import_deck_cards::ImportDeckCardsResult},
    },
    http::contracts::deck_card::HttpImportDeckCards,
};

/// Trait for importing cards into a deck from plain text.
#[allow(missing_docs)]
pub trait ClientImportDeckCards {
    fn import_deck_cards(
        &self,
        deck_id: Uuid,
        text: &str,
        board: Option<&str>,
        mode: ImportMode,
        session: &Session,
    ) -> impl Future<Output = Result<ImportDeckCardsResult, ApiError>> + Send;
}

impl ClientImportDeckCards for ZwipeClient {
    async fn import_deck_cards(
        &self,
        deck_id: Uuid,
        text: &str,
        board: Option<&str>,
        mode: ImportMode,
        session: &Session,
    ) -> Result<ImportDeckCardsResult, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&import_deck_cards_route(deck_id));

        let body = HttpImportDeckCards {
            text: text.to_string(),
            board: board.map(|b| b.to_string()),
            mode,
        };
        info!("POST {} body: {:?}", url, body);

        let response = self
            .client
            .post(url)
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
