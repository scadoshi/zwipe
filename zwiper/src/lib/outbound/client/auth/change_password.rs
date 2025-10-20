use std::future::Future;

use reqwest::StatusCode;
use thiserror::Error;
use zwipe::inbound::http::{
    handlers::auth::change_password::HttpChangePassword, routes::change_password_route,
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

pub trait ChangePassword {
    fn change_password(
        &self,
        request: HttpChangePassword,
    ) -> impl Future<Output = Result<(), ChangePasswordError>> + Send;
}

impl ChangePassword for AuthClient {
    async fn change_password(
        &self,
        request: HttpChangePassword,
    ) -> Result<(), ChangePasswordError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&change_password_route());
        let response = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
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
