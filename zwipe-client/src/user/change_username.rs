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
        let body = serde_json::to_value(&request)?;
        self.call(ChangeUsername(body), Some(session)).await
    }
}
