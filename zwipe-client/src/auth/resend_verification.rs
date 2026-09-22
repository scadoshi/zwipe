//! Resend email verification API client.

use crate::{ClientError, ZwipeClient};
use zwipe_core::{
    domain::auth::models::session::Session, http::endpoints::auth::ResendVerification,
};

impl ZwipeClient {
    /// Re-sends the email verification link for the authenticated user.
    pub async fn resend_verification(&self, session: &Session) -> Result<(), ClientError> {
        self.call(ResendVerification, Some(session)).await
    }
}
