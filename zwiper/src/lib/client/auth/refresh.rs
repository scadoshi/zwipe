use std::future::Future;

use crate::client::auth::AuthClient;
use reqwest::StatusCode;
use thiserror::Error;
use zwipe::{
    domain::auth::models::session::Session,
    inbound::http::{
        handlers::auth::refresh_session::HttpRefreshSession, routes::refresh_session_route,
    },
};

#[derive(Debug, Error)]
pub enum RefreshError {
    #[error("invalid credentials")]
    Unauthorized,
    #[error("access forbidden")]
    Forbidden,
    #[error("something went wrong")]
    SomethingWentWrong,
    #[error("network error")]
    Network(reqwest::Error),
    #[error("{0}")]
    InvalidRequest(String),
}

impl From<reqwest::Error> for RefreshError {
    fn from(value: reqwest::Error) -> Self {
        Self::Network(value)
    }
}

impl From<serde_json::Error> for RefreshError {
    fn from(_value: serde_json::Error) -> Self {
        Self::SomethingWentWrong
    }
}

pub trait Refresh {
    fn refresh(
        request: &HttpRefreshSession,
        auth_client: &AuthClient,
    ) -> impl Future<Output = Result<Session, RefreshError>> + Send;
}

impl Refresh for AuthClient {
    async fn refresh(
        request: &HttpRefreshSession,
        auth_client: &AuthClient,
    ) -> Result<Session, RefreshError> {
        let mut url = auth_client.app_config.backend_url.clone();
        url.set_path(&refresh_session_route());

        let response = auth_client
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&request)?)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let success: Session = response.json().await?;
                Ok(success)
            }
            StatusCode::UNAUTHORIZED => Err(RefreshError::Unauthorized),
            StatusCode::FORBIDDEN => Err(RefreshError::Forbidden),
            StatusCode::UNPROCESSABLE_ENTITY => {
                let message = response.text().await?;
                Err(RefreshError::InvalidRequest(message))
            }
            _ => Err(RefreshError::SomethingWentWrong),
        }
    }
}
