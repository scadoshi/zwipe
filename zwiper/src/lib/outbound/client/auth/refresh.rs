//! Access token refresh API client.

use crate::outbound::client::{ClientError, ZwipeClient};
use zwipe_core::{
    domain::auth::models::session::Session,
    http::{contracts::auth::HttpRefreshSession, endpoints::auth::Refresh},
};

impl ZwipeClient {
    /// Refreshes access tokens using a refresh token.
    pub async fn refresh(&self, request: &HttpRefreshSession) -> Result<Session, ClientError> {
        let body = serde_json::to_value(request)?;
        self.call(Refresh(body), None).await
    }
}
