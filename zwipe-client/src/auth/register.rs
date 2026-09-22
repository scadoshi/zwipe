//! New user registration API client.

use crate::{ClientError, ZwipeClient};
use zwipe_core::{
    domain::auth::models::session::Session,
    http::{contracts::auth::HttpRegisterUser, endpoints::auth::Register},
};

impl ZwipeClient {
    /// Registers new user accounts.
    pub async fn register(&self, request: HttpRegisterUser) -> Result<Session, ClientError> {
        let body = serde_json::to_value(&request)?;
        self.call(Register(body), None).await
    }
}
