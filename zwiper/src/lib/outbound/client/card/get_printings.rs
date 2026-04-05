//! Fetch all printings of a card by oracle ID.

use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use uuid::Uuid;
use zwipe::inbound::http::{routes::get_printings_route, ApiError};
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;

/// Trait for fetching all printings of a card by oracle ID.
#[allow(missing_docs)]
pub trait ClientGetPrintings {
    fn get_printings(
        &self,
        oracle_id: Uuid,
        session: &Session,
    ) -> impl Future<Output = Result<Vec<Card>, ApiError>> + Send;
}

impl ClientGetPrintings for ZwipeClient {
    async fn get_printings(&self, oracle_id: Uuid, session: &Session) -> Result<Vec<Card>, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&get_printings_route(oracle_id));

        let response = self
            .client
            .get(url)
            .bearer_auth(&*session.access_token.value)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let cards: Vec<Card> = response.json().await?;
                Ok(cards)
            }
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
