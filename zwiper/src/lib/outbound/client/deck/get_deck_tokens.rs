//! Fetch tokens produced by a deck's cards.

use crate::outbound::client::{ClientError, ZwipeClient};
use reqwest::StatusCode;
use std::future::Future;
use tracing::info;
use uuid::Uuid;
use zwipe::inbound::http::routes::get_deck_tokens_route;
use zwipe_core::domain::{auth::models::session::Session, card::Card};

/// Trait for fetching all token cards produced by a deck.
#[allow(missing_docs)]
pub trait ClientGetDeckTokens {
    fn get_deck_tokens(
        &self,
        deck_id: Uuid,
        session: &Session,
    ) -> impl Future<Output = Result<Vec<Card>, ClientError>> + Send;
}

impl ClientGetDeckTokens for ZwipeClient {
    async fn get_deck_tokens(
        &self,
        deck_id: Uuid,
        session: &Session,
    ) -> Result<Vec<Card>, ClientError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&get_deck_tokens_route(deck_id));
        info!("GET {}", url);

        let response = self
            .client
            .get(url)
            .bearer_auth(&*session.access_token.value)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let tokens: Vec<Card> = response.json().await?;
                Ok(tokens)
            }
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
