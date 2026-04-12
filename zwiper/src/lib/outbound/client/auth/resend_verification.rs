//! Resend email verification API client.

use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use tracing::info;
use zwipe::inbound::http::{routes::resend_verification_route, ApiError};
use zwipe_core::domain::auth::models::session::Session;

/// Trait for re-sending the email verification link for the authenticated user.
#[allow(missing_docs)]
pub trait ClientResendEmailVerification {
    fn resend_verification(
        &self,
        session: &Session,
    ) -> impl Future<Output = Result<(), ApiError>> + Send;
}

impl ClientResendEmailVerification for ZwipeClient {
    async fn resend_verification(&self, session: &Session) -> Result<(), ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&resend_verification_route());
        info!("POST {}", url);
        let response = self
            .client
            .post(url)
            .bearer_auth(&*session.access_token.value)
            .send()
            .await?;
        match response.status() {
            StatusCode::OK => Ok(()),
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
