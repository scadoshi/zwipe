//! Change username endpoint.

use crate::{ClientError, ZwipeClient};
use zwipe_core::{
    domain::{auth::models::session::Session, user::User},
    http::{contracts::auth::HttpChangeUsername, endpoints::user::ChangeUsername},
};

impl ZwipeClient {
    /// Updates usernames.
    pub async fn change_username(
        &self,
        request: HttpChangeUsername,
        session: &Session,
    ) -> Result<User, ClientError> {
        self.call(ChangeUsername(request), Some(session)).await
    }
}
