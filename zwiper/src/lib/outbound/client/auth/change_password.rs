use reqwest::StatusCode;
use std::future::Future;
use thiserror::Error;
use zwipe::{
    domain::auth::models::session::Session,
    inbound::http::{
        handlers::auth::change_password::HttpChangePassword, routes::change_password_route,
    },
};

use crate::outbound::client::auth::AuthClient;

#[derive(Debug, Error)]
pub enum ChangePasswordError {
    #[error("invalid credentials")]
    Unauthorized,
    #[error("something went wrong")]
    SomethingWentWrong,
    #[error("network error")]
    Network(reqwest::Error),
    #[error("{0}")]
    InvalidRequest(String),
    #[error("session expired")]
    SessionExpired,
}

impl From<reqwest::Error> for ChangePasswordError {
    fn from(value: reqwest::Error) -> Self {
        Self::Network(value)
    }
}

impl From<serde_json::Error> for ChangePasswordError {
    fn from(_value: serde_json::Error) -> Self {
        Self::SomethingWentWrong
    }
}

pub trait AuthClientChangePassword {
    fn change_password(
        &self,
        request: HttpChangePassword,
        session: &Session,
    ) -> impl Future<Output = Result<(), ChangePasswordError>> + Send;
}

impl AuthClientChangePassword for AuthClient {
    async fn change_password(
        &self,
        request: HttpChangePassword,
        session: &Session,
    ) -> Result<(), ChangePasswordError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&change_password_route());
        let response = self
            .client
            .put(url)
            .header("Content-Type", "application/json")
            .header(
                "Authorization",
                format!("Bearer {}", session.access_token.value.as_str()),
            )
            .body(serde_json::to_string(&request)?)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => Ok(()),
            StatusCode::UNPROCESSABLE_ENTITY => {
                let message = response.text().await?;
                Err(ChangePasswordError::InvalidRequest(message))
            }
            StatusCode::UNAUTHORIZED => Err(ChangePasswordError::Unauthorized),
            _ => Err(ChangePasswordError::SomethingWentWrong),
        }
    }
}
