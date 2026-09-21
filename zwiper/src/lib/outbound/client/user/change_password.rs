//! Change user password endpoint.

use crate::outbound::client::{ClientError, ZwipeClient};
use std::future::Future;
use zwipe_core::{
    domain::auth::models::session::Session,
    http::{contracts::auth::HttpChangePassword, endpoints::user::ChangePassword},
};

/// Trait for updating user passwords.
#[allow(missing_docs)]
pub trait ClientChangePassword {
    fn change_password(
        &self,
        request: HttpChangePassword,
        session: &Session,
    ) -> impl Future<Output = Result<(), ClientError>> + Send;
}

impl ClientChangePassword for ZwipeClient {
    async fn change_password(
        &self,
        request: HttpChangePassword,
        session: &Session,
    ) -> Result<(), ClientError> {
        let body = serde_json::to_value(&request)?;
        self.call(ChangePassword(body), Some(session)).await
    }
}
