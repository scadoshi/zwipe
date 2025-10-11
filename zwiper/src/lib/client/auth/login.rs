use std::future::Future;

use crate::client::auth::AuthClient;
use reqwest::StatusCode;
use thiserror::Error;
use zwipe::{
    domain::auth::models::session::Session,
    inbound::http::{handlers::auth::HttpAuthenticateUser, routes::login_route},
};

#[derive(Debug, Error)]
pub enum LoginError {
    #[error("invalid credentials")]
    Unauthorized,
    #[error("something went wrong")]
    SomethingWentWrong,
    #[error("network error")]
    Network(reqwest::Error),
    #[error("{0}")]
    InvalidRequest(String),
}

impl From<reqwest::Error> for LoginError {
    fn from(value: reqwest::Error) -> Self {
        Self::Network(value)
    }
}

impl From<serde_json::Error> for LoginError {
    fn from(_value: serde_json::Error) -> Self {
        Self::SomethingWentWrong
    }
}

pub trait Login {
    fn authenticate_user(
        &self,
        request: HttpAuthenticateUser,
    ) -> impl Future<Output = Result<Session, LoginError>> + Send;
}

impl Login for AuthClient {
    async fn authenticate_user(
        &self,
        request: HttpAuthenticateUser,
    ) -> Result<Session, LoginError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&login_route());
        let response = self
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
            StatusCode::UNPROCESSABLE_ENTITY => {
                let message = response.text().await?;
                Err(LoginError::InvalidRequest(message))
            }
            StatusCode::UNAUTHORIZED => Err(LoginError::Unauthorized),
            _ => Err(LoginError::SomethingWentWrong),
        }
    }
}
