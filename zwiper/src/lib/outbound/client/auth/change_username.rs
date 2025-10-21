use std::future::Future;

use reqwest::StatusCode;
use thiserror::Error;
use zwipe::{
    domain::{auth::models::session::Session, user::models::User},
    inbound::http::{
        handlers::auth::change_username::HttpChangeUsername, routes::change_username_route,
    },
};

use crate::outbound::client::auth::AuthClient;
#[derive(Debug, Error)]
pub enum ChangeUsernameError {
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

impl From<reqwest::Error> for ChangeUsernameError {
    fn from(value: reqwest::Error) -> Self {
        Self::Network(value)
    }
}

impl From<serde_json::Error> for ChangeUsernameError {
    fn from(_value: serde_json::Error) -> Self {
        Self::SomethingWentWrong
    }
}

pub trait AuthClientChangeUsername {
    fn change_username(
        &self,
        request: HttpChangeUsername,
        session: &Session,
    ) -> impl Future<Output = Result<User, ChangeUsernameError>> + Send;
}

impl AuthClientChangeUsername for AuthClient {
    async fn change_username(
        &self,
        request: HttpChangeUsername,
        session: &Session,
    ) -> Result<User, ChangeUsernameError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&change_username_route());
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
            StatusCode::OK => {
                let success: User = response.json().await?;
                Ok(success)
            }
            StatusCode::UNPROCESSABLE_ENTITY => {
                let message = response.text().await?;
                Err(ChangeUsernameError::InvalidRequest(message))
            }
            StatusCode::UNAUTHORIZED => Err(ChangeUsernameError::Unauthorized),
            _ => Err(ChangeUsernameError::SomethingWentWrong),
        }
    }
}
