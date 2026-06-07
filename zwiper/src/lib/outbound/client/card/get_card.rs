//! Fetch a single card by ID.

use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use tracing::info;
use uuid::Uuid;
use zwipe::inbound::http::{routes::get_card_route, ApiError};
use zwipe_core::domain::card::Card;

/// Trait for fetching a single card by its Scryfall data ID.
#[allow(missing_docs)]
pub trait ClientGetCard {
    fn get_card(
        &self,
        scryfall_data_id: Uuid,
    ) -> impl Future<Output = Result<Card, ApiError>> + Send;
}

impl ClientGetCard for ZwipeClient {
    async fn get_card(&self, scryfall_data_id: Uuid) -> Result<Card, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&get_card_route(scryfall_data_id));
        info!("GET {}", url);

        let response = self.client.get(url).send().await?;

        match response.status() {
            StatusCode::OK => {
                let card: Card = response.json().await?;
                Ok(card)
            }
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
