//! Forgot password API client.

use crate::outbound::client::{ClientError, ZwipeClient};
use std::future::Future;
use zwipe_core::http::{
    contracts::auth::HttpRequestPasswordReset, endpoints::auth::RequestPasswordReset,
};

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
        let body = serde_json::to_value(&request)?;
        self.call(RequestPasswordReset(body), None).await
    }
}
