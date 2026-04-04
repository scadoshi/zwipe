//! Fetch a single card by ID.

use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use uuid::Uuid;
use zwipe::{
    domain::card::models::Card,
    inbound::http::{routes::get_card_route, ApiError},
};
use zwipe_core::domain::auth::models::session::Session;

/// Trait for fetching a single card by its Scryfall data ID.
#[allow(missing_docs)]
pub trait ClientGetCard {
    fn get_card(
        &self,
        scryfall_data_id: Uuid,
        session: &Session,
    ) -> impl Future<Output = Result<Card, ApiError>> + Send;
}

impl ClientGetCard for ZwipeClient {
    async fn get_card(&self, scryfall_data_id: Uuid, session: &Session) -> Result<Card, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&get_card_route(scryfall_data_id));

        let response = self
            .client
            .get(url)
            .bearer_auth(&*session.access_token.value)
            .send()
            .await?;

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
