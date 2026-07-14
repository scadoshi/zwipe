//! User login API client.

use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use tracing::info;
use zwipe::inbound::http::{ApiError, routes::login_route};
use zwipe_core::{
    domain::auth::models::{platform::ClientPlatform, session::Session},
    http::contracts::auth::HttpAuthenticateUser,
};

/// Trait for authenticating users via the login endpoint.
#[allow(missing_docs)]
pub trait ClientLogin {
    fn authenticate_user(
        &self,
        request: HttpAuthenticateUser,
    ) -> impl Future<Output = Result<Session, ApiError>> + Send;
}

impl ClientLogin for ZwipeClient {
    async fn authenticate_user(&self, request: HttpAuthenticateUser) -> Result<Session, ApiError> {
        let mut request = request;
        request.platform = Some(ClientPlatform::CURRENT);
        request.client_version = Some(env!("CARGO_PKG_VERSION").to_string());

        let mut url = self.app_config.backend_url.clone();
        url.set_path(&login_route());
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
