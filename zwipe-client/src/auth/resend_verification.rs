//! Resend email verification API client.

use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::endpoints::auth::ResendVerification;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Re-sends the email verification link for the authenticated user.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn resend_verification(&self, session: &Session) -> Result<(), ClientError> {
        self.call(ResendVerification, Some(session)).await
    }
}
