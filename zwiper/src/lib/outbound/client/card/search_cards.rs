//! Card search with filters.

use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use tracing::info;
use zwipe::inbound::http::{ApiError, routes::search_cards_route};
use zwipe_core::domain::{
    auth::models::session::Session,
    card::{Card, search_card::card_filter::CardQuery},
};

/// Trait for searching cards with filter criteria.
#[allow(missing_docs)]
pub trait ClientSearchCards {
    fn search_cards(
        &self,
        card_filter: &CardQuery,
        session: &Session,
    ) -> impl Future<Output = Result<Vec<Card>, ApiError>> + Send;
}

impl ClientSearchCards for ZwipeClient {
    async fn search_cards(
        &self,
        card_filter: &CardQuery,
        session: &Session,
    ) -> Result<Vec<Card>, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&search_cards_route());

        info!("POST {} filter: {:?}", url, card_filter);

        let response = self
            .client
            .post(url)
            .json(card_filter)
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
