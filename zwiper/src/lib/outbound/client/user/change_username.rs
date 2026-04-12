//! Change username endpoint.

use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use tracing::info;
use zwipe::inbound::http::{routes::change_username_route, ApiError};
use zwipe_core::http::contracts::auth::HttpChangeUsername;
use zwipe_core::domain::{auth::models::session::Session, user::User};

/// Trait for updating usernames.
#[allow(missing_docs)]
pub trait ClientChangeUsername {
    fn change_username(
        &self,
        request: HttpChangeUsername,
        session: &Session,
    ) -> impl Future<Output = Result<User, ApiError>> + Send;
}

impl ClientChangeUsername for ZwipeClient {
    async fn change_username(
        &self,
        request: HttpChangeUsername,
        session: &Session,
    ) -> Result<User, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&change_username_route());
        info!("PUT {}", url);
        let response = self
            .client
            .put(url)
            .json(&request)
            .bearer_auth(&*session.access_token.value)
            .send()
            .await?;

        let status = response.status();

        match status {
            StatusCode::OK => {
                let updated: User = response.json().await?;
                Ok(updated)
            }
            _ => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
