//! Share / unshare a deck (public link token management).

use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use tracing::info;
use uuid::Uuid;
use zwipe::inbound::http::{ApiError, routes::share_deck_route};
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::deck::HttpDeckShareToken;

/// Trait for creating and revoking a deck's public share link.
#[allow(missing_docs)]
pub trait ClientShareDeck {
    /// Generates (or regenerates) the share token. Re-sharing rotates it, so
    /// any previously shared link dies.
    fn share_deck(
        &self,
        deck_id: Uuid,
        session: &Session,
    ) -> impl Future<Output = Result<HttpDeckShareToken, ApiError>> + Send;

    /// Revokes the deck's share token (stops sharing).
    fn unshare_deck(
        &self,
        deck_id: Uuid,
        session: &Session,
    ) -> impl Future<Output = Result<(), ApiError>> + Send;
}

impl ClientShareDeck for ZwipeClient {
    async fn share_deck(
        &self,
        deck_id: Uuid,
        session: &Session,
    ) -> Result<HttpDeckShareToken, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&share_deck_route(deck_id));
        info!("POST {}", url);

        let response = self
            .client
            .post(url)
            .bearer_auth(&*session.access_token.value)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => Ok(response.json::<HttpDeckShareToken>().await?),
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }

    async fn unshare_deck(&self, deck_id: Uuid, session: &Session) -> Result<(), ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&share_deck_route(deck_id));
        info!("DELETE {}", url);

        let response = self
            .client
            .delete(url)
            .bearer_auth(&*session.access_token.value)
            .send()
            .await?;

        match response.status() {
            StatusCode::NO_CONTENT => Ok(()),
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
