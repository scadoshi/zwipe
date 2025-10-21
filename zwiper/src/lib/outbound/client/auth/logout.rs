use reqwest::StatusCode;
use std::future::Future;
use thiserror::Error;
use zwipe::{domain::auth::models::session::Session, inbound::http::routes::logout_route};

use crate::outbound::client::auth::AuthClient;

#[derive(Debug, Error)]
pub enum LogoutError {
    #[error("unauthorized")]
    Unauthorized,
    #[error("something went wrong")]
    SomethingWentWrong,
    #[error("network error")]
    Network(reqwest::Error),
}

impl From<reqwest::Error> for LogoutError {
    fn from(value: reqwest::Error) -> Self {
        Self::Network(value)
    }
}

pub trait Logout {
    fn logout(&self, session: &Session) -> impl Future<Output = Result<(), LogoutError>> + Send;
}

impl Logout for AuthClient {
    async fn logout(&self, session: &Session) -> Result<(), LogoutError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&logout_route());

        let response = self
            .client
            .post(url)
            .header(
                "Authorization",
                format!("Bearer {}", session.access_token.value.as_str()),
            )
            .send()
            .await?;

        match response.status() {
            StatusCode::NO_CONTENT | StatusCode::OK => Ok(()),
            StatusCode::UNAUTHORIZED => Err(LogoutError::Unauthorized),
            _ => Err(LogoutError::SomethingWentWrong),
        }
    }
}
