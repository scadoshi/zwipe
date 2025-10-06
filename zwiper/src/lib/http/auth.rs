use crate::config::AppConfig;
use reqwest::{Client, StatusCode};
use thiserror::Error;
use zwipe::{
    domain::auth::models::{
        AuthenticateUser, AuthenticateUserSuccess, InvalidAuthenticateUser, InvalidRawRegisterUser,
        RawRegisterUser,
    },
    inbound::http::handlers::auth::{HttpAuthenticateUser, HttpRegisterUser},
};

#[derive(Debug, Clone)]
pub struct AuthClient {
    client: Client,
    app_config: AppConfig,
}

impl Default for AuthClient {
    fn default() -> Self {
        let app_config = AppConfig::default();
        let client = Client::new();
        Self { client, app_config }
    }
}

// ==========
//  register
// ==========

#[derive(Debug, Error)]
pub enum RegisterUserError {
    #[error("something went wrong")]
    SomethingWentWrong,
    #[error("network error")]
    Network(reqwest::Error),
    #[error("invalid request: {0}")]
    InvalidRequest(String),
}

impl From<reqwest::Error> for RegisterUserError {
    fn from(value: reqwest::Error) -> Self {
        Self::Network(value)
    }
}

impl From<serde_json::Error> for RegisterUserError {
    fn from(_value: serde_json::Error) -> Self {
        Self::SomethingWentWrong
    }
}

// might not need this at all
pub fn validate_register_user(
    username: &str,
    email: &str,
    password: &str,
) -> Result<HttpRegisterUser, InvalidRawRegisterUser> {
    RawRegisterUser::new(username, email, password).map(|r| r.into())
}

pub async fn register_user(
    request: HttpRegisterUser,
    auth_client: &AuthClient,
) -> Result<AuthenticateUserSuccess, RegisterUserError> {
    let mut url = auth_client.app_config.backend_url.clone();
    url.set_path("/api/auth/register");
    let response = auth_client
        .client
        .post(url)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&request)?)
        .send()
        .await?;

    match response.status() {
        StatusCode::CREATED => {
            let success: AuthenticateUserSuccess = response.json().await?;
            Ok(success)
        }
        StatusCode::UNPROCESSABLE_ENTITY => {
            let message = response.text().await?;
            Err(RegisterUserError::InvalidRequest(message))
        }
        _ => Err(RegisterUserError::SomethingWentWrong),
    }
}

// ==============
//  authenticate
// ==============

#[derive(Debug, Error)]
pub enum AuthenticateUserError {
    #[error("invalid credentials")]
    Unauthorized,
    #[error("something went wrong")]
    SomethingWentWrong,
    #[error("network error")]
    Network(reqwest::Error),
    #[error("invalid request: {0}")]
    InvalidRequest(String),
}

impl From<reqwest::Error> for AuthenticateUserError {
    fn from(value: reqwest::Error) -> Self {
        Self::Network(value)
    }
}

impl From<serde_json::Error> for AuthenticateUserError {
    fn from(_value: serde_json::Error) -> Self {
        Self::SomethingWentWrong
    }
}

// might not need this at all
// as each part is being validated separately (e.g. username, email, etc.)
// so those errors can be placed around the ui as needed
// but this might be a good final boss
// think about it
pub fn validate_authenticate_user(
    identifier: &str,
    password: &str,
) -> Result<HttpAuthenticateUser, InvalidAuthenticateUser> {
    AuthenticateUser::new(identifier, password).map(|r| r.into())
}

pub async fn authenticate_user(
    request: HttpAuthenticateUser,
    auth_client: &AuthClient,
) -> Result<AuthenticateUserSuccess, AuthenticateUserError> {
    let mut url = auth_client.app_config.backend_url.clone();
    url.set_path("/api/auth/login");
    let response = auth_client
        .client
        .post(url)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&request)?)
        .send()
        .await?;

    match response.status() {
        StatusCode::OK => {
            let success: AuthenticateUserSuccess = response.json().await?;
            Ok(success)
        }
        StatusCode::UNPROCESSABLE_ENTITY => {
            let message = response.text().await?;
            Err(AuthenticateUserError::InvalidRequest(message))
        }
        StatusCode::UNAUTHORIZED => Err(AuthenticateUserError::Unauthorized),
        _ => Err(AuthenticateUserError::SomethingWentWrong),
    }
}
