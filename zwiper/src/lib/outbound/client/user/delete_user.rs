//! Delete user account endpoint.

use crate::outbound::client::{ClientError, ZwipeClient};
use std::future::Future;
use zwipe_core::{
    domain::auth::models::session::Session,
    http::{contracts::auth::HttpDeleteUser, endpoints::user::DeleteUser},
};

/// Trait for deleting user accounts.
#[allow(missing_docs)]
pub trait ClientDeleteUser {
    fn delete_user(
        &self,
        request: HttpDeleteUser,
        session: &Session,
    ) -> impl Future<Output = Result<(), ClientError>> + Send;
}

impl ClientDeleteUser for ZwipeClient {
    async fn delete_user(
        &self,
        request: HttpDeleteUser,
        session: &Session,
    ) -> Result<(), ClientError> {
        let body = serde_json::to_value(&request)?;
        self.call(DeleteUser(body), Some(session)).await
    }
}
