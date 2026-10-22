//! Change user email endpoint.

use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::user::User;
use zwipe_core::http::contracts::auth::HttpChangeEmail;
use zwipe_core::http::endpoints::user::ChangeEmail;

use crate::{ClientError, ZwipeClient};

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
