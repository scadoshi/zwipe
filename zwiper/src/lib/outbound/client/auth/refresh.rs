//! Access token refresh API client.

use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use tracing::info;
use zwipe::inbound::http::{ApiError, routes::refresh_session_route};
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::auth::HttpRefreshSession;

/// Trait for refreshing access tokens using a refresh token.
#[allow(missing_docs)]
pub trait ClientRefresh {
    fn refresh(
        &self,
        request: &HttpRefreshSession,
    ) -> impl Future<Output = Result<Session, ApiError>> + Send;
}

impl ClientRefresh for ZwipeClient {
    async fn refresh(&self, request: &HttpRefreshSession) -> Result<Session, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&refresh_session_route());
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
