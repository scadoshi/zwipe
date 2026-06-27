//! Delete user account endpoint.

use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use tracing::info;
use zwipe::inbound::http::{ApiError, routes::delete_user_route};
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::auth::HttpDeleteUser;

/// Trait for deleting user accounts.
#[allow(missing_docs)]
pub trait ClientDeleteUser {
    fn delete_user(
        &self,
        request: HttpDeleteUser,
        session: &Session,
    ) -> impl Future<Output = Result<(), ApiError>> + Send;
}

impl ClientDeleteUser for ZwipeClient {
    async fn delete_user(
        &self,
        request: HttpDeleteUser,
        session: &Session,
    ) -> Result<(), ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&delete_user_route());
        info!("DELETE {}", url);
        let response = self
            .client
            .delete(url)
            .json(&request)
            .bearer_auth(&*session.access_token.value)
            .send()
            .await?;

        let status = response.status();

        match status {
            StatusCode::NO_CONTENT => Ok(()),
            _ => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
