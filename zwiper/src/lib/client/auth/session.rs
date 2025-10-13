use std::future::Future;

use crate::client::auth::{
    refresh::{Refresh, RefreshError},
    AuthClient,
};
use zwipe::{
    domain::auth::models::session::Session,
    inbound::http::{
        handlers::auth::refresh_session::HttpRefreshSession, routes::refresh_session_route,
    },
};

pub struct ActiveSession(Session);

impl ActiveSession {
    pub fn session(&self) -> Session {
        self.0.clone()
    }
}

pub trait GetActiveSession {
    fn get_active_session(
        &self,
        session: Session,
    ) -> impl Future<Output = Result<Option<ActiveSession>, RefreshError>> + Send;
}

impl GetActiveSession for AuthClient {
    async fn get_active_session(
        &self,
        session: Session,
    ) -> Result<Option<ActiveSession>, RefreshError> {
        if session.is_expired() {
            return Ok(None);
        }

        if session.access_token.is_expired() {
            let mut url = self.app_config.backend_url.clone();
            url.set_path(&refresh_session_route());

            let request = HttpRefreshSession::from(&session);

            let session = self.refresh(&request).await?;

            return Ok(Some(ActiveSession(session)));
        }

        Ok(Some(ActiveSession(session)))
    }
}
