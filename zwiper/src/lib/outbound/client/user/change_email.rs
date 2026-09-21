//! Change user email endpoint.

use crate::outbound::client::{ClientError, ZwipeClient};
use std::future::Future;
use zwipe_core::{
    domain::{auth::models::session::Session, user::User},
    http::{contracts::auth::HttpChangeEmail, endpoints::user::ChangeEmail},
};

/// Trait for updating user email addresses.
#[allow(missing_docs)]
pub trait ClientChangeEmail {
    fn change_email(
        &self,
        request: HttpChangeEmail,
        session: &Session,
    ) -> impl Future<Output = Result<User, ClientError>> + Send;
}

impl ClientChangeEmail for ZwipeClient {
    async fn change_email(
        &self,
        request: HttpChangeEmail,
        session: &Session,
    ) -> Result<User, ClientError> {
        let body = serde_json::to_value(&request)?;
        self.call(ChangeEmail(body), Some(session)).await
    }
}
