//! Change user email endpoint.

use crate::{ClientError, ZwipeClient};
use zwipe_core::{
    domain::{auth::models::session::Session, user::User},
    http::{contracts::auth::HttpChangeEmail, endpoints::user::ChangeEmail},
};

impl ZwipeClient {
    /// Updates user email addresses.
    pub async fn change_email(
        &self,
        request: HttpChangeEmail,
        session: &Session,
    ) -> Result<User, ClientError> {
        let body = serde_json::to_value(&request)?;
        self.call(ChangeEmail(body), Some(session)).await
    }
}
