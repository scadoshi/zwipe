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
        self.call(ChangeEmail(request), Some(session)).await
    }
}
