//! New user registration API client.

use crate::outbound::client::{ClientError, ZwipeClient};
use std::future::Future;
use zwipe_core::{
    domain::auth::models::session::Session,
    http::{contracts::auth::HttpRegisterUser, endpoints::auth::Register},
};

/// Trait for registering new user accounts.
#[allow(missing_docs)]
pub trait ClientRegister {
    fn register(
        &self,
        request: HttpRegisterUser,
    ) -> impl Future<Output = Result<Session, ClientError>> + Send;
}

impl ClientRegister for ZwipeClient {
    async fn register(&self, request: HttpRegisterUser) -> Result<Session, ClientError> {
        let body = serde_json::to_value(&request)?;
        self.call(Register(body), None).await
    }
}
