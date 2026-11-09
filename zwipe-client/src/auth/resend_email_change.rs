//! Resend a pending email change's link API client.

use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::endpoints::auth::ResendEmailChange;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Mails a fresh link for the authenticated user's pending email change,
    /// replacing the old link and its expiry.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete or the server answers with
    /// an error status: 404 when no change is pending, 429 within a minute of the last resend.
    pub async fn resend_email_change(&self, session: &Session) -> Result<(), ClientError> {
        self.call(ResendEmailChange, Some(session)).await
    }
}
