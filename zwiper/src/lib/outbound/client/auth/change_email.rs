use std::future::Future;

use reqwest::StatusCode;
use thiserror::Error;
use zwipe::{
    domain::user::models::User,
    inbound::http::{handlers::auth::change_email::HttpChangeEmail, routes::change_email_route},
};

use crate::outbound::client::auth::AuthClient;
#[derive(Debug, Error)]
pub enum ChangeEmailError {
    #[error("invalid credentials")]
    Unauthorized,
    #[error("something went wrong")]
    SomethingWentWrong,
    #[error("network error")]
    Network(reqwest::Error),
    #[error("{0}")]
    InvalidRequest(String),
}

impl From<reqwest::Error> for ChangeEmailError {
    fn from(value: reqwest::Error) -> Self {
        Self::Network(value)
    }
}

impl From<serde_json::Error> for ChangeEmailError {
    fn from(_value: serde_json::Error) -> Self {
        Self::SomethingWentWrong
    }
}

pub trait ChangeEmail {
    fn change_email(
        &self,
        request: HttpChangeEmail,
    ) -> impl Future<Output = Result<User, ChangeEmailError>> + Send;
}

impl ChangeEmail for AuthClient {
    async fn change_email(&self, request: HttpChangeEmail) -> Result<User, ChangeEmailError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&change_email_route());
        let response = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
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
                Err(ChangeEmailError::InvalidRequest(message))
            }
            StatusCode::UNAUTHORIZED => Err(ChangeEmailError::Unauthorized),
            _ => Err(ChangeEmailError::SomethingWentWrong),
        }
    }
}
