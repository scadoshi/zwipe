//! Fetch the deck-tag catalog.

use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use tracing::info;
use zwipe::inbound::http::{ApiError, routes::get_deck_tags_route};
use zwipe_core::domain::{auth::models::session::Session, deck::DeckTagView};

/// Trait for fetching the full deck-tag catalog (slug, label, description, seed
/// otags). Authenticated — it lives under the deck routes; the deck-tag picker
/// (an authed flow) is its consumer.
#[allow(missing_docs)]
pub trait ClientGetDeckTags {
    fn get_deck_tags(
        &self,
        session: &Session,
    ) -> impl Future<Output = Result<Vec<DeckTagView>, ApiError>> + Send;
}

impl ClientGetDeckTags for ZwipeClient {
    async fn get_deck_tags(&self, session: &Session) -> Result<Vec<DeckTagView>, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&get_deck_tags_route());
        info!("GET {}", url);

        let response = self
            .client
            .get(url)
            .bearer_auth(&*session.access_token.value)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let tags: Vec<DeckTagView> = response.json().await?;
                Ok(tags)
            }
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
