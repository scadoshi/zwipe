//! Deck-aware card search.
//!
//! Same `CardFilter` body as the plain search, but scoped to a deck: the
//! server excludes cards already in the deck (any board, plus profile slots)
//! and defaults to synergy ordering when no explicit sort is set.

use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use tracing::info;
use uuid::Uuid;
use zwipe::inbound::http::{ApiError, routes::search_deck_cards_route};
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::{Card, search_card::card_filter::CardFilter};

/// Trait for deck-aware card search.
#[allow(missing_docs)]
pub trait ClientSearchDeckCards {
    fn search_deck_cards(
        &self,
        deck_id: Uuid,
        card_filter: &CardFilter,
        session: &Session,
    ) -> impl Future<Output = Result<Vec<Card>, ApiError>> + Send;
}

impl ClientSearchDeckCards for ZwipeClient {
    async fn search_deck_cards(
        &self,
        deck_id: Uuid,
        card_filter: &CardFilter,
        session: &Session,
    ) -> Result<Vec<Card>, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&search_deck_cards_route(deck_id));

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
