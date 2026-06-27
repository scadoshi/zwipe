//! Remove a card from a deck.

use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use tracing::info;
use uuid::Uuid;
use zwipe::inbound::http::{ApiError, routes::delete_deck_card_route};
use zwipe_core::domain::auth::models::session::Session;

/// Trait for removing cards from a deck.
#[allow(missing_docs)]
pub trait ClientDeleteDeckCard {
    fn delete_deck_card(
        &self,
        deck_id: Uuid,
        scryfall_data_id: Uuid,
        session: &Session,
    ) -> impl Future<Output = Result<(), ApiError>> + Send;
}

impl ClientDeleteDeckCard for ZwipeClient {
    async fn delete_deck_card(
        &self,
        deck_id: Uuid,
        scryfall_data_id: Uuid,
        session: &Session,
    ) -> Result<(), ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&delete_deck_card_route(deck_id, scryfall_data_id));
        info!("DELETE {}", url);

        let response = self
            .client
            .delete(url)
            .bearer_auth(&*session.access_token.value)
            .send()
            .await?;

        match response.status() {
            StatusCode::NO_CONTENT | StatusCode::OK => Ok(()),
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
