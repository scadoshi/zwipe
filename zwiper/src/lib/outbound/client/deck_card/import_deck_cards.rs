//! Import cards into a deck from plain-text decklist.

use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use uuid::Uuid;
use zwipe::{
    domain::{
        auth::models::session::Session,
        deck::models::deck_card::import_deck_cards::ImportDeckCardsResult,
    },
    inbound::http::{
        handlers::deck_card::import_deck_cards::HttpImportDeckCards,
        routes::import_deck_cards_route, ApiError,
    },
};

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
