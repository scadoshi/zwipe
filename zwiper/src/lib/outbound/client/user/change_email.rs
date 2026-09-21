//! Change user email endpoint.

use crate::outbound::client::{ClientError, ZwipeClient};
use reqwest::StatusCode;
use std::future::Future;
use tracing::info;
use zwipe_core::{
    domain::{auth::models::session::Session, user::User},
    http::{contracts::auth::HttpChangeEmail, paths::CHANGE_EMAIL_ROUTE},
};

/// Trait for updating user email addresses.
#[allow(missing_docs)]
pub trait ClientChangeEmail {
    fn change_email(
        &self,
        request: HttpChangeEmail,
        session: &Session,
    ) -> impl Future<Output = Result<User, ClientError>> + Send;
}

impl ClientChangeEmail for ZwipeClient {
    async fn change_email(
        &self,
        request: HttpChangeEmail,
        session: &Session,
    ) -> Result<User, ClientError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(CHANGE_EMAIL_ROUTE);
        info!("PATCH {}", url);
        let response = self
            .client
            .patch(url)
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
