use std::future::Future;

use crate::outbound::client::auth::AuthClient;
use reqwest::StatusCode;
use thiserror::Error;
use zwipe::{
    domain::auth::models::session::Session,
    inbound::http::{handlers::auth::register_user::HttpRegisterUser, routes::register_route},
};

#[derive(Debug, Error)]
pub enum RegisterError {
    #[error("something went wrong")]
    SomethingWentWrong,
    #[error("network error")]
    Network(reqwest::Error),
    #[error("{0}")]
    InvalidRequest(String),
}

impl From<reqwest::Error> for RegisterError {
    fn from(value: reqwest::Error) -> Self {
        Self::Network(value)
    }
}

impl From<serde_json::Error> for RegisterError {
    fn from(_value: serde_json::Error) -> Self {
        Self::SomethingWentWrong
    }
}

pub trait Register {
    fn register(
        &self,
        request: HttpRegisterUser,
    ) -> impl Future<Output = Result<Session, RegisterError>> + Send;
}

impl Register for AuthClient {
    async fn register(&self, request: HttpRegisterUser) -> Result<Session, RegisterError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&register_route());
        let response = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&request)?)
            .send()
            .await?;

        match response.status() {
            StatusCode::CREATED => {
                let success: Session = response.json().await?;
                Ok(success)
            }
            StatusCode::UNPROCESSABLE_ENTITY => {
                let message = response.text().await?;
                Err(RegisterError::InvalidRequest(message))
            }
            _ => Err(RegisterError::SomethingWentWrong),
        }
    }
}
