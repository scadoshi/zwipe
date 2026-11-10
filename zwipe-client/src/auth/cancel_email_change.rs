//! Cancel a pending email change API client.

use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::endpoints::auth::CancelEmailChange;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Drops the authenticated user's pending email change, so its link no
    /// longer confirms.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete or the server answers with
    /// an error status: 404 when no change is pending.
    pub async fn cancel_email_change(&self, session: &Session) -> Result<(), ClientError> {
        self.call(CancelEmailChange, Some(session)).await
    }
}
