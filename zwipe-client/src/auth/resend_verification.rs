//! Resend email verification API client.

use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::endpoints::auth::ResendVerification;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Re-sends the email verification link for the authenticated user.
    pub async fn resend_verification(&self, session: &Session) -> Result<(), ClientError> {
        self.call(ResendVerification, Some(session)).await
    }
}
