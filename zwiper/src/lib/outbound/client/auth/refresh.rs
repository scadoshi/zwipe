//! Access token refresh API client.

use crate::outbound::client::{ClientError, ZwipeClient};
use std::future::Future;
use zwipe_core::{
    domain::auth::models::session::Session,
    http::{contracts::auth::HttpRefreshSession, endpoints::auth::Refresh},
};

/// Trait for refreshing access tokens using a refresh token.
#[allow(missing_docs)]
pub trait ClientRefresh {
    fn refresh(
        &self,
        request: &HttpRefreshSession,
    ) -> impl Future<Output = Result<Session, ClientError>> + Send;
}

impl ClientRefresh for ZwipeClient {
    async fn refresh(&self, request: &HttpRefreshSession) -> Result<Session, ClientError> {
        let body = serde_json::to_value(request)?;
        self.call(Refresh(body), None).await
    }
}
