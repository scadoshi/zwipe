//! Fetch user profile endpoint.

use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::user::User;
use zwipe_core::http::endpoints::user::GetUser;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Fetches user profile data.
    pub async fn get_user(&self, session: &Session) -> Result<User, ClientError> {
        self.call(GetUser, Some(session)).await
    }
}
