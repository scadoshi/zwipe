//! User logout API client.

use crate::outbound::client::{ClientError, ZwipeClient};
use std::future::Future;
use zwipe_core::{domain::auth::models::session::Session, http::endpoints::auth::Logout};

/// Trait for logging out users and invalidating sessions.
#[allow(missing_docs)]
pub trait ClientLogout {
    fn logout(&self, session: &Session) -> impl Future<Output = Result<(), ClientError>> + Send;
}

impl ClientLogout for ZwipeClient {
    async fn logout(&self, session: &Session) -> Result<(), ClientError> {
        self.call(Logout, Some(session)).await
    }
}
