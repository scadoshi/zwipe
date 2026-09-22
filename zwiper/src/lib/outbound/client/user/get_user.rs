//! Fetch user profile endpoint.

use crate::outbound::client::{ClientError, ZwipeClient};
use zwipe_core::{
    domain::{auth::models::session::Session, user::User},
    http::endpoints::user::GetUser,
};

impl ZwipeClient {
    /// Fetches user profile data.
    pub async fn get_user(&self, session: &Session) -> Result<User, ClientError> {
        self.call(GetUser, Some(session)).await
    }
}
