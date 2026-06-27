//! Clone an existing deck.

use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use tracing::info;
use uuid::Uuid;
use zwipe::inbound::http::{ApiError, routes::clone_deck_route};
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::deck::{HttpCloneDeck, HttpClonedDeck};

/// Trait for cloning an existing deck into a new one with a caller-chosen name.
#[allow(missing_docs)]
pub trait ClientCloneDeck {
    fn clone_deck(
        &self,
        source_deck_id: Uuid,
        body: &HttpCloneDeck,
        session: &Session,
    ) -> impl Future<Output = Result<HttpClonedDeck, ApiError>> + Send;
}

impl ClientCloneDeck for ZwipeClient {
    async fn clone_deck(
        &self,
        source_deck_id: Uuid,
        body: &HttpCloneDeck,
        session: &Session,
    ) -> Result<HttpClonedDeck, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&clone_deck_route(source_deck_id));
        info!("POST {} body: {:?}", url, body);

        let response = self
            .client
            .post(url)
            .json(&body)
            .bearer_auth(&*session.access_token.value)
            .send()
            .await?;

        match response.status() {
            StatusCode::CREATED => Ok(response.json::<HttpClonedDeck>().await?),
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
