//! Change user password endpoint.

use crate::outbound::client::{ClientError, ZwipeClient};
use reqwest::StatusCode;
use std::future::Future;
use tracing::info;
use zwipe_core::{
    domain::auth::models::session::Session,
    http::{contracts::auth::HttpChangePassword, paths::CHANGE_PASSWORD_ROUTE},
};

/// Trait for updating user passwords.
#[allow(missing_docs)]
pub trait ClientChangePassword {
    fn change_password(
        &self,
        request: HttpChangePassword,
        session: &Session,
    ) -> impl Future<Output = Result<(), ClientError>> + Send;
}

impl ClientChangePassword for ZwipeClient {
    async fn change_password(
        &self,
        request: HttpChangePassword,
        session: &Session,
    ) -> Result<(), ClientError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(CHANGE_PASSWORD_ROUTE);
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
            StatusCode::OK => Ok(()),
            _ => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
