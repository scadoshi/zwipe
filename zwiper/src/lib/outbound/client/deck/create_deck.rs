//! Create new deck.

use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use tracing::info;
use zwipe::inbound::http::{ApiError, routes::create_deck_route};
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::deck::deck_profile::DeckProfile;
use zwipe_core::http::contracts::deck::HttpCreateDeckProfile;

/// Trait for creating new deck profiles.
#[allow(missing_docs)]
pub trait ClientCreateDeck {
    fn create_deck_profile(
        &self,
        request: &HttpCreateDeckProfile,
        session: &Session,
    ) -> impl Future<Output = Result<DeckProfile, ApiError>> + Send;
}

impl ClientCreateDeck for ZwipeClient {
    async fn create_deck_profile(
        &self,
        request: &HttpCreateDeckProfile,
        session: &Session,
    ) -> Result<DeckProfile, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&create_deck_route());
        info!("POST {} body: {:?}", url, request);

        let response = self
            .client
            .post(url)
            .json(request)
            .bearer_auth(&*session.access_token.value)
            .send()
            .await?;

        match response.status() {
            StatusCode::CREATED => {
                let new: DeckProfile = response.json().await?;
                Ok(new)
            }
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
