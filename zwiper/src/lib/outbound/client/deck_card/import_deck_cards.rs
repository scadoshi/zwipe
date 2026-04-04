//! Import cards into a deck from plain-text decklist.

use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use uuid::Uuid;
use zwipe::inbound::http::{routes::import_deck_cards_route, ApiError};
use zwipe_core::http::contracts::deck_card::HttpImportDeckCards;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::deck::requests::import_deck_cards::ImportDeckCardsResult;

/// Trait for importing cards into a deck from plain text.
#[allow(missing_docs)]
pub trait ClientImportDeckCards {
    fn import_deck_cards(
        &self,
        deck_id: Uuid,
        text: &str,
        session: &Session,
    ) -> impl Future<Output = Result<ImportDeckCardsResult, ApiError>> + Send;
}

impl ClientImportDeckCards for ZwipeClient {
    async fn import_deck_cards(
        &self,
        deck_id: Uuid,
        text: &str,
        session: &Session,
    ) -> Result<ImportDeckCardsResult, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&import_deck_cards_route(deck_id));

        let body = HttpImportDeckCards {
            text: text.to_string(),
        };

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
