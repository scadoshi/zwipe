//! User logout API client.

use crate::outbound::client::{ClientError, ZwipeClient};
use zwipe_core::{domain::auth::models::session::Session, http::endpoints::auth::Logout};

impl ZwipeClient {
    /// Logs out users and invalidating sessions.
    pub async fn logout(&self, session: &Session) -> Result<(), ClientError> {
        self.call(Logout, Some(session)).await
    }
}
