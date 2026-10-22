//! User logout API client.

use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::endpoints::auth::Logout;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Logs out users and invalidating sessions.
    pub async fn logout(&self, session: &Session) -> Result<(), ClientError> {
        self.call(Logout, Some(session)).await
    }
}
