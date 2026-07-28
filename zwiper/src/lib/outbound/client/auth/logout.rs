//! User logout API client.

use crate::outbound::client::{ClientError, ZwipeClient};
use reqwest::StatusCode;
use std::future::Future;
use tracing::info;
use zwipe::inbound::http::routes::logout_route;
use zwipe_core::domain::auth::models::session::Session;

/// Trait for logging out users and invalidating sessions.
#[allow(missing_docs)]
pub trait ClientLogout {
    fn logout(&self, session: &Session) -> impl Future<Output = Result<(), ClientError>> + Send;
}

impl ClientLogout for ZwipeClient {
    async fn logout(&self, session: &Session) -> Result<(), ClientError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&logout_route());
        info!("POST {}", url);

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
