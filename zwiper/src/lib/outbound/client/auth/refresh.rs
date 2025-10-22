use crate::outbound::client::{auth::AuthClient, error::ApiError};
use reqwest::StatusCode;
use std::future::Future;
use zwipe::{
    domain::auth::models::session::Session,
    inbound::http::{
        handlers::auth::refresh_session::HttpRefreshSession, routes::refresh_session_route,
    },
};

pub trait AuthClientRefresh {
    fn refresh(
        &self,
        request: &HttpRefreshSession,
    ) -> impl Future<Output = Result<Session, ApiError>> + Send;
}

impl AuthClientRefresh for AuthClient {
    async fn refresh(&self, request: &HttpRefreshSession) -> Result<Session, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&refresh_session_route());

        let response = self.client.post(url).json(&request).send().await?;

        let status = response.status();

        match status {
            StatusCode::OK => {
                let new: Session = response.json().await?;
                Ok(new)
            }
            _ => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
