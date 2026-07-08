//! Deck-aware card search.
//!
//! Same `CardQuery` body as the plain search, but scoped to a deck: the
//! server excludes cards already in the deck (any board, plus profile slots)
//! and defaults to synergy ordering when no explicit sort is set.

use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use tracing::info;
use uuid::Uuid;
use zwipe::inbound::http::{ApiError, routes::search_deck_cards_route};
use zwipe_core::domain::{
    auth::models::session::Session,
    card::{Card, search_card::card_filter::CardQuery},
};

/// Trait for deck-aware card search.
#[allow(missing_docs)]
pub trait ClientSearchDeckCards {
    fn search_deck_cards(
        &self,
        deck_id: Uuid,
        card_filter: &CardQuery,
        session: &Session,
    ) -> impl Future<Output = Result<(Vec<Card>, bool), ApiError>> + Send;
}

impl ClientSearchDeckCards for ZwipeClient {
    /// Returns `(cards, synergy_warming)` — `synergy_warming` is true when
    /// synergy was requested but the commander's cache was still warming, so the
    /// server served the full pool (signalled via the `x-synergy-applied` header).
    async fn search_deck_cards(
        &self,
        deck_id: Uuid,
        card_filter: &CardQuery,
        session: &Session,
    ) -> Result<(Vec<Card>, bool), ApiError> {
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
                // `x-synergy-applied: false` means synergy was requested but the
                // commander's cache is still warming, so the full pool was served.
                // Read the header before `json()` consumes the response.
                let synergy_warming = response
                    .headers()
                    .get("x-synergy-applied")
                    .and_then(|v| v.to_str().ok())
                    .map(|v| v == "false")
                    .unwrap_or(false);
                let cards: Vec<Card> = response.json().await?;
                Ok((cards, synergy_warming))
            }
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
