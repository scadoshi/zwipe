//! Check a reset link's token before the reset form shows.

use zwipe_core::http::contracts::auth::HttpCheckResetToken;
use zwipe_core::http::endpoints::auth::CheckResetToken;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Checks that the token from a reset link is still live, without
    /// spending or extending it.
    ///
    /// Unauthenticated, for the same reason as [`ZwipeClient::verify_email`].
    ///
    /// # Errors
    ///
    /// Returns [`ClientError::BadRequest`] when the token is unknown, used or
    /// expired. Any other [`ClientError`] (a request that fails to complete,
    /// another error status, or a reply that doesn't decode) says nothing
    /// about the token.
    pub async fn check_reset_token(&self, token: String) -> Result<(), ClientError> {
        self.call(CheckResetToken(HttpCheckResetToken { token }), None)
            .await
    }
}
