//! Update a deck card (PATCH — idempotent, absolute quantity).
//!
//! Speaks the PATCH wire shape ([`HttpPatchDeckCard`]): `quantity` sets an
//! absolute value, so retries/replays are harmless. The server's legacy PUT
//! delta route still exists for older shipped clients — migration:
//! `context/plans/patch_idempotent_updates.md`.

use crate::outbound::client::{ClientError, ZwipeClient};
use reqwest::StatusCode;
use std::future::Future;
use tracing::info;
use uuid::Uuid;
use zwipe::inbound::http::routes::update_deck_card_route;
use zwipe_core::{
    domain::{auth::models::session::Session, deck::deck_card::DeckCard},
    http::contracts::deck_card::HttpPatchDeckCard,
};

/// Trait for updating a card in a deck.
#[allow(missing_docs)]
pub trait ClientUpdateDeckCard {
    fn update_deck_card(
        &self,
        deck_id: Uuid,
        scryfall_data_id: Uuid,
        request: &HttpPatchDeckCard,
        session: &Session,
    ) -> impl Future<Output = Result<DeckCard, ClientError>> + Send;
}

impl ClientUpdateDeckCard for ZwipeClient {
    async fn update_deck_card(
        &self,
        deck_id: Uuid,
        scryfall_data_id: Uuid,
        request: &HttpPatchDeckCard,
        session: &Session,
    ) -> Result<DeckCard, ClientError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&update_deck_card_route(deck_id, scryfall_data_id));
        info!("PATCH {} body: {:?}", url, request);

        let response = self
            .client
            .patch(url)
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
