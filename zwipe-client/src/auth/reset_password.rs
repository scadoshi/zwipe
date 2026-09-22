//! Finish a password reset from the reset link.

use crate::{ClientError, ZwipeClient};
use zwipe_core::{
    domain::auth::models::secret::Secret,
    http::{contracts::auth::HttpResetPassword, endpoints::auth::ResetPassword},
};

impl ZwipeClient {
    /// Sets a new password using the token from the reset link.
    ///
    /// Unauthenticated, for the same reason as [`ZwipeClient::verify_email`].
    /// The password is validated server-side; callers should run
    /// `zwipe_core`'s password validation first so the user sees the rule
    /// they broke without a round trip.
    pub async fn reset_password(
        &self,
        token: String,
        new_password: Secret,
    ) -> Result<(), ClientError> {
        let body = serde_json::to_value(HttpResetPassword {
            token,
            new_password,
        })?;
        self.call(ResetPassword(body), None).await
    }
}
