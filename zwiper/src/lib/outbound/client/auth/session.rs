use std::future::Future;

use crate::outbound::client::auth::{
    refresh::{AuthClientRefresh, RefreshError},
    AuthClient,
};
use zwipe::{
    domain::auth::models::session::Session,
    inbound::http::{
        handlers::auth::refresh_session::HttpRefreshSession, routes::refresh_session_route,
    },
};

pub trait AuthClientSession {
    fn get_active_session(
        &self,
        session: &Session,
    ) -> impl Future<Output = Result<Option<Session>, RefreshError>> + Send;

    fn infallible_get_active_session(
        &self,
        session: &Session,
    ) -> impl Future<Output = Option<Session>> + Send;
}

impl AuthClientSession for AuthClient {
    async fn get_active_session(&self, session: &Session) -> Result<Option<Session>, RefreshError> {
        if session.is_expired() {
            return Ok(None);
        }

        if session.access_token.is_expired() {
            let mut url = self.app_config.backend_url.clone();
            url.set_path(&refresh_session_route());
            let request = HttpRefreshSession::from(session);
            let session = self.refresh(&request).await?;
            return Ok(Some(session));
        }

        Ok(Some(session.clone()))
    }

    async fn infallible_get_active_session(&self, session: &Session) -> Option<Session> {
        match self.get_active_session(session).await {
            Err(e) => {
                tracing::error!("failed to get active session: {e}");
                None
            }
            Ok(Some(session)) => Some(session),
            Ok(None) => None,
        }
    }
}
