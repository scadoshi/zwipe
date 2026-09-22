//! Access token refresh API client.

use crate::{ClientError, ZwipeClient};
use zwipe_core::{
    domain::auth::models::session::Session,
    http::{contracts::auth::HttpRefreshSession, endpoints::auth::Refresh},
};

impl ZwipeClient {
    /// Refreshes access tokens using a refresh token.
    pub async fn refresh(&self, request: &HttpRefreshSession) -> Result<Session, ClientError> {
        self.call(Refresh(request.clone()), None).await
    }
}
