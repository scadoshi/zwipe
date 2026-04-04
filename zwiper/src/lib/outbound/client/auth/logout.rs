//! User logout API client.

use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use zwipe::inbound::http::{routes::logout_route, ApiError};
use zwipe_core::domain::auth::models::session::Session;

/// Trait for logging out users and invalidating sessions.
#[allow(missing_docs)]
pub trait ClientLogout {
    fn logout(&self, session: &Session) -> impl Future<Output = Result<(), ApiError>> + Send;
}

impl ClientLogout for ZwipeClient {
    async fn logout(&self, session: &Session) -> Result<(), ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&logout_route());

        let response = self
            .client
            .post(url)
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
