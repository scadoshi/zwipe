//! Forgot password API client.

use crate::outbound::client::{ClientError, ZwipeClient};
use reqwest::StatusCode;
use std::future::Future;
use tracing::info;
use zwipe::inbound::http::routes::forgot_password_route;
use zwipe_core::http::contracts::auth::HttpRequestPasswordReset;

/// Trait for initiating a password reset via the forgot-password endpoint.
#[allow(missing_docs)]
pub trait ClientForgotPassword {
    fn request_password_reset(
        &self,
        request: HttpRequestPasswordReset,
    ) -> impl Future<Output = Result<(), ClientError>> + Send;
}

impl ClientForgotPassword for ZwipeClient {
    async fn request_password_reset(
        &self,
        request: HttpRequestPasswordReset,
    ) -> Result<(), ClientError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&forgot_password_route());
        info!("POST {}", url);

        let response = self.client.post(url).json(&request).send().await?;

        match response.status() {
            StatusCode::OK => Ok(()),
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
