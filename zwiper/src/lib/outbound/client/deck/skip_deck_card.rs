//! Post a single durable skip (and its undo) for a deck.

use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use tracing::info;
use uuid::Uuid;
use zwipe::inbound::http::{
    ApiError,
    routes::{skip_deck_card_route, unskip_deck_card_route},
};
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::deck::HttpSkipDeckCard;

/// Trait for posting and undoing a single deck-card skip.
#[allow(missing_docs)]
pub trait ClientSkipDeckCard {
    fn skip_deck_card(
        &self,
        deck_id: Uuid,
        oracle_id: Uuid,
        session: &Session,
    ) -> impl Future<Output = Result<(), ApiError>> + Send;

    fn unskip_deck_card(
        &self,
        deck_id: Uuid,
        oracle_id: Uuid,
        session: &Session,
    ) -> impl Future<Output = Result<(), ApiError>> + Send;
}

impl ClientSkipDeckCard for ZwipeClient {
    async fn skip_deck_card(
        &self,
        deck_id: Uuid,
        oracle_id: Uuid,
        session: &Session,
    ) -> Result<(), ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&skip_deck_card_route(deck_id));
        info!("POST {}", url);

        let response = self
            .client
            .post(url)
            .bearer_auth(&*session.access_token.value)
            .json(&HttpSkipDeckCard { oracle_id })
            .send()
            .await?;

        match response.status() {
            StatusCode::NO_CONTENT => Ok(()),
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }

    async fn unskip_deck_card(
        &self,
        deck_id: Uuid,
        oracle_id: Uuid,
        session: &Session,
    ) -> Result<(), ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&unskip_deck_card_route(deck_id, oracle_id));
        info!("DELETE {}", url);

        let response = self
            .client
            .delete(url)
            .bearer_auth(&*session.access_token.value)
            .send()
            .await?;

        match response.status() {
            StatusCode::NO_CONTENT => Ok(()),
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
