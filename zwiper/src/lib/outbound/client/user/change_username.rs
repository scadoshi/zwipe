//! Change username endpoint.

use crate::outbound::client::{ClientError, ZwipeClient};
use std::future::Future;
use zwipe_core::{
    domain::{auth::models::session::Session, user::User},
    http::{contracts::auth::HttpChangeUsername, endpoints::user::ChangeUsername},
};

/// Trait for updating usernames.
#[allow(missing_docs)]
pub trait ClientChangeUsername {
    fn change_username(
        &self,
        request: HttpChangeUsername,
        session: &Session,
    ) -> impl Future<Output = Result<User, ClientError>> + Send;
}

impl ClientChangeUsername for ZwipeClient {
    async fn change_username(
        &self,
        request: HttpChangeUsername,
        session: &Session,
    ) -> Result<User, ClientError> {
        let body = serde_json::to_value(&request)?;
        self.call(ChangeUsername(body), Some(session)).await
    }
}
