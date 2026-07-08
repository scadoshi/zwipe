//! Change user email endpoint.

use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use tracing::info;
use zwipe::inbound::http::{ApiError, routes::change_email_route};
use zwipe_core::{
    domain::{auth::models::session::Session, user::User},
    http::contracts::auth::HttpChangeEmail,
};

/// Trait for updating user email addresses.
#[allow(missing_docs)]
pub trait ClientChangeEmail {
    fn change_email(
        &self,
        request: HttpChangeEmail,
        session: &Session,
    ) -> impl Future<Output = Result<User, ApiError>> + Send;
}

impl ClientChangeEmail for ZwipeClient {
    async fn change_email(
        &self,
        request: HttpChangeEmail,
        session: &Session,
    ) -> Result<User, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&change_email_route());
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
