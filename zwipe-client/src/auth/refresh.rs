//! Access token refresh API client.

use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::auth::HttpRefreshSession;
use zwipe_core::http::endpoints::auth::Refresh;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Refreshes access tokens using a refresh token.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn refresh(&self, request: &HttpRefreshSession) -> Result<Session, ClientError> {
        self.call(Refresh(request.clone()), None).await
    }
}
