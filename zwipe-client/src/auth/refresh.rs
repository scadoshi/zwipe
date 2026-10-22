//! Access token refresh API client.

use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::auth::HttpRefreshSession;
use zwipe_core::http::endpoints::auth::Refresh;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Refreshes access tokens using a refresh token.
    pub async fn refresh(&self, request: &HttpRefreshSession) -> Result<Session, ClientError> {
        self.call(Refresh(request.clone()), None).await
    }
}
