//! Access token refresh API client.

use crate::outbound::client::{ClientError, ZwipeClient};
use reqwest::StatusCode;
use std::future::Future;
use tracing::info;
use zwipe_core::{
    domain::auth::models::session::Session,
    http::{contracts::auth::HttpRefreshSession, paths::REFRESH_SESSION_ROUTE},
};

/// Trait for refreshing access tokens using a refresh token.
#[allow(missing_docs)]
pub trait ClientRefresh {
    fn refresh(
        &self,
        request: &HttpRefreshSession,
    ) -> impl Future<Output = Result<Session, ClientError>> + Send;
}

impl ClientRefresh for ZwipeClient {
    async fn refresh(&self, request: &HttpRefreshSession) -> Result<Session, ClientError> {
        // Re-send the running app version each refresh so the rotated session
        // records the live version, not the one it was first created with.
        let mut request = HttpRefreshSession::new(&request.user_id, &request.refresh_token);
        request.client_version = Some(env!("CARGO_PKG_VERSION").to_string());

        let mut url = self.app_config.backend_url.clone();
        url.set_path(REFRESH_SESSION_ROUTE);
        info!("POST {}", url);

        let response = self.client.post(url).json(&request).send().await?;

        match response.status() {
            StatusCode::OK => {
                let new: Session = response.json().await?;
                Ok(new)
            }
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
