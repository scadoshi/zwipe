//! Change username endpoint.

use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::user::User;
use zwipe_core::http::contracts::auth::HttpChangeUsername;
use zwipe_core::http::endpoints::user::ChangeUsername;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Updates usernames.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn change_username(
        &self,
        request: HttpChangeUsername,
        session: &Session,
    ) -> Result<User, ClientError> {
        self.call(ChangeUsername(request), Some(session)).await
    }
}
