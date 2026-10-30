//! Forgot password API client.

use zwipe_core::http::contracts::auth::HttpRequestPasswordReset;
use zwipe_core::http::endpoints::auth::RequestPasswordReset;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Initiates a password reset via the forgot-password endpoint.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn request_password_reset(
        &self,
        request: HttpRequestPasswordReset,
    ) -> Result<(), ClientError> {
        self.call(RequestPasswordReset(request), None).await
    }
}
