//! Update card quantity in a deck.

use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use uuid::Uuid;
use zwipe::inbound::http::{routes::update_deck_card_route, ApiError};
use zwipe_core::http::contracts::deck_card::HttpUpdateDeckCard;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::deck::deck_card::DeckCard;

/// Trait for updating card quantity in a deck.
#[allow(missing_docs)]
pub trait ClientUpdateDeckCard {
    fn update_deck_card(
        &self,
        deck_id: Uuid,
        scryfall_data_id: Uuid,
        request: &HttpUpdateDeckCard,
        session: &Session,
    ) -> impl Future<Output = Result<DeckCard, ApiError>> + Send;
}

impl ClientUpdateDeckCard for ZwipeClient {
    async fn update_deck_card(
        &self,
        deck_id: Uuid,
        scryfall_data_id: Uuid,
        request: &HttpUpdateDeckCard,
        session: &Session,
    ) -> Result<DeckCard, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&update_deck_card_route(deck_id, scryfall_data_id));

        let response = self
            .client
            .put(url)
            .json(request)
            .bearer_auth(&*session.access_token.value)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let updated: DeckCard = response.json().await?;
                Ok(updated)
            }
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
