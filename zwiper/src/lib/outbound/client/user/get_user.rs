//! Fetch user profile endpoint.

use crate::outbound::client::{ClientError, ZwipeClient};
use std::future::Future;
use zwipe_core::{
    domain::{auth::models::session::Session, user::User},
    http::endpoints::user::GetUser,
};

/// Trait for fetching user profile data.
#[allow(missing_docs)]
pub trait ClientGetUser {
    fn get_user(&self, session: &Session)
    -> impl Future<Output = Result<User, ClientError>> + Send;
}

impl ClientGetUser for ZwipeClient {
    async fn get_user(&self, session: &Session) -> Result<User, ClientError> {
        self.call(GetUser, Some(session)).await
    }
}
