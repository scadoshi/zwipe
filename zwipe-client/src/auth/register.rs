//! New user registration API client.

use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::auth::HttpRegisterUser;
use zwipe_core::http::endpoints::auth::Register;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Registers new user accounts.
    pub async fn register(&self, request: HttpRegisterUser) -> Result<Session, ClientError> {
        self.call(Register(request), None).await
    }
}
