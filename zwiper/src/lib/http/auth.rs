use std::sync::OnceLock;

use crate::config::AppConfig;
use reqwest::{Client, StatusCode};
use thiserror::Error;
use zwipe::{
    domain::auth::models::session::{
        InvalidRefreshSession, InvalidRevokeSessions, RevokeSessions, Session,
    },
    inbound::http::{
        handlers::auth::{HttpAuthenticateUser, HttpRefreshSession, HttpRegisterUser},
        routes::{login_route, logout_route, refresh_session_route, register_route},
    },
};

#[derive(Debug, Clone)]
pub struct AuthClient {
    client: Client,
    app_config: AppConfig,
}

impl AuthClient {
    pub fn new() -> Self {
        static CONFIG: OnceLock<AppConfig> = OnceLock::new();
        let app_config = CONFIG
            .get_or_init(|| AppConfig::from_env().expect("failed to initialize app config"))
            .clone();
        Self {
            client: Client::new(),
            app_config,
        }
    }
}

// ==========
//  register
// ==========

#[derive(Debug, Error)]
pub enum ResisterError {
    #[error("something went wrong")]
    SomethingWentWrong,
    #[error("network error")]
    Network(reqwest::Error),
    #[error("{0}")]
    InvalidRequest(String),
}

impl From<reqwest::Error> for ResisterError {
    fn from(value: reqwest::Error) -> Self {
        Self::Network(value)
    }
}

impl From<serde_json::Error> for ResisterError {
    fn from(_value: serde_json::Error) -> Self {
        Self::SomethingWentWrong
    }
}

pub async fn register(
    request: HttpRegisterUser,
    auth_client: &AuthClient,
) -> Result<Session, ResisterError> {
    let mut url = auth_client.app_config.backend_url.clone();
    url.set_path(&register_route());
    let response = auth_client
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
            Err(ResisterError::InvalidRequest(message))
        }
        _ => Err(ResisterError::SomethingWentWrong),
    }
}

// ==============
//  authenticate
// ==============

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

pub async fn login(
    request: HttpAuthenticateUser,
    auth_client: &AuthClient,
) -> Result<Session, LoginError> {
    let mut url = auth_client.app_config.backend_url.clone();
    url.set_path(&login_route());
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
        StatusCode::UNPROCESSABLE_ENTITY => {
            let message = response.text().await?;
            Err(LoginError::InvalidRequest(message))
        }
        StatusCode::UNAUTHORIZED => Err(LoginError::Unauthorized),
        _ => Err(LoginError::SomethingWentWrong),
    }
}

// =========
//  refresh
// =========

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

pub async fn refresh(
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

// ========
//  logout
// ========

#[derive(Debug, Error)]
pub enum LogoutError {
    #[error("thing")]
    Thing,
}

pub async fn logout(auth_client: &AuthClient) -> Result<(), LogoutError> {
    let mut url = auth_client.app_config.backend_url.clone();
    url.set_path(&logout_route());

    // x
    todo!()
}
