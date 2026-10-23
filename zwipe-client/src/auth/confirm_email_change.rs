//! Switch to a pending email address from the link sent to it.

use zwipe_core::http::contracts::auth::HttpConfirmEmailChange;
use zwipe_core::http::endpoints::auth::ConfirmEmailChange;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Switches the account to its pending address with the token from the
    /// confirmation link.
    ///
    /// Unauthenticated, for the same reason as [`ZwipeClient::verify_email`].
    /// A token that is unknown, spent or expired answers 422; an address
    /// another account took in the meantime answers 409.
    pub async fn confirm_email_change(&self, token: String) -> Result<(), ClientError> {
        self.call(ConfirmEmailChange(HttpConfirmEmailChange { token }), None)
            .await
    }
}
