//! Resend email verification API client.

use crate::outbound::client::{ClientError, ZwipeClient};
use std::future::Future;
use zwipe_core::{
    domain::auth::models::session::Session, http::endpoints::auth::ResendVerification,
};

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
        self.call(ResendVerification, Some(session)).await
    }
}
