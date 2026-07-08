//! Clear a deck's suppression set (skipped/removed cards).

use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use tracing::info;
use uuid::Uuid;
use zwipe::inbound::http::{ApiError, routes::clear_deck_suppressions_route};
use zwipe_core::{
    domain::auth::models::session::Session, http::contracts::deck::HttpClearedSuppressions,
};

/// Trait for clearing a deck's suppression set.
#[allow(missing_docs)]
pub trait ClientClearDeckSuppressions {
    fn clear_deck_suppressions(
        &self,
        deck_id: Uuid,
        session: &Session,
    ) -> impl Future<Output = Result<HttpClearedSuppressions, ApiError>> + Send;
}

impl ClientClearDeckSuppressions for ZwipeClient {
    async fn clear_deck_suppressions(
        &self,
        deck_id: Uuid,
        session: &Session,
    ) -> Result<HttpClearedSuppressions, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&clear_deck_suppressions_route(deck_id));
        info!("DELETE {}", url);

        let response = self
            .client
            .delete(url)
            .bearer_auth(&*session.access_token.value)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => Ok(response.json().await?),
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
