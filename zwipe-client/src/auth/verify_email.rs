//! Confirm an email address from the verification link.

use crate::{ClientError, ZwipeClient};
use zwipe_core::http::{contracts::auth::HttpVerifyEmail, endpoints::auth::VerifyEmail};

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
