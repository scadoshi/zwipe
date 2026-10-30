//! User logout API client.

use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::endpoints::auth::Logout;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Logs out users and invalidating sessions.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn logout(&self, session: &Session) -> Result<(), ClientError> {
        self.call(Logout, Some(session)).await
    }
}
