//! Change user password endpoint.

use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::auth::HttpChangePassword;
use zwipe_core::http::endpoints::user::ChangePassword;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Updates user passwords.
    pub async fn change_password(
        &self,
        request: HttpChangePassword,
        session: &Session,
    ) -> Result<(), ClientError> {
        self.call(ChangePassword(request), Some(session)).await
    }
}
