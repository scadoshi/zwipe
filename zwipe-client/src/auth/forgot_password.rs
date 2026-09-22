//! Forgot password API client.

use crate::{ClientError, ZwipeClient};
use zwipe_core::http::{
    contracts::auth::HttpRequestPasswordReset, endpoints::auth::RequestPasswordReset,
};

impl ZwipeClient {
    /// Initiates a password reset via the forgot-password endpoint.
    pub async fn request_password_reset(
        &self,
        request: HttpRequestPasswordReset,
    ) -> Result<(), ClientError> {
        self.call(RequestPasswordReset(request), None).await
    }
}
