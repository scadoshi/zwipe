//! New user registration API client.

use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::auth::{HttpRegisterUser, HttpRegistrationAccepted};
use zwipe_core::http::endpoints::auth::{Register, RegisterVerifyFirst};

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Registers new user accounts.
    pub async fn register(&self, request: HttpRegisterUser) -> Result<Session, ClientError> {
        self.call(Register(request), None).await
    }

    /// Creates an account without a session. The answer is the same whether
    /// or not the email already has an account; the user logs in afterwards.
    pub async fn register_verify_first(
        &self,
        mut request: HttpRegisterUser,
    ) -> Result<HttpRegistrationAccepted, ClientError> {
        request.verify_first = true;
        self.call(RegisterVerifyFirst(request), None).await
    }
}
