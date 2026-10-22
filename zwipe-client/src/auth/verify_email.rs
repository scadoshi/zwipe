//! Confirm an email address from the verification link.

use zwipe_core::http::contracts::auth::HttpVerifyEmail;
use zwipe_core::http::endpoints::auth::VerifyEmail;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Confirms an email address with the token from the verification link.
    ///
    /// Unauthenticated: the link opens in a browser with no session. A token
    /// that is unknown, spent or expired answers 4xx.
    pub async fn verify_email(&self, token: String) -> Result<(), ClientError> {
        self.call(VerifyEmail(HttpVerifyEmail { token }), None)
            .await
    }
}
