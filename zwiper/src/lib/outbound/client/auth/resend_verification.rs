//! Resend email verification API client.

use crate::outbound::client::{ClientError, ZwipeClient};
use reqwest::StatusCode;
use std::future::Future;
use tracing::info;
use zwipe_core::{domain::auth::models::session::Session, http::paths::RESEND_VERIFICATION_ROUTE};

/// Trait for re-sending the email verification link for the authenticated user.
#[allow(missing_docs)]
pub trait ClientResendEmailVerification {
    fn resend_verification(
        &self,
        session: &Session,
    ) -> impl Future<Output = Result<(), ClientError>> + Send;
}

impl ClientResendEmailVerification for ZwipeClient {
    async fn resend_verification(&self, session: &Session) -> Result<(), ClientError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(RESEND_VERIFICATION_ROUTE);
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
