//! Forgot password API client.

use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use zwipe::inbound::http::{
    handlers::auth::request_password_reset::HttpRequestPasswordReset,
    routes::forgot_password_route,
    ApiError,
};

/// Trait for initiating a password reset via the forgot-password endpoint.
#[allow(missing_docs)]
pub trait ClientForgotPassword {
    fn request_password_reset(
        &self,
        request: HttpRequestPasswordReset,
    ) -> impl Future<Output = Result<(), ApiError>> + Send;
}

impl ClientForgotPassword for ZwipeClient {
    async fn request_password_reset(&self, request: HttpRequestPasswordReset) -> Result<(), ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&forgot_password_route());

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
