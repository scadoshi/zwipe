//! New user registration API client.

use zwipe_core::http::contracts::auth::{HttpRegisterUser, HttpRegistrationAccepted};
use zwipe_core::http::endpoints::auth::Register;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Creates an account without a session. The answer is the same whether
    /// or not the email already has an account; the user logs in afterwards.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn register(
        &self,
        request: HttpRegisterUser,
    ) -> Result<HttpRegistrationAccepted, ClientError> {
        self.call(Register(request), None).await
    }
}
