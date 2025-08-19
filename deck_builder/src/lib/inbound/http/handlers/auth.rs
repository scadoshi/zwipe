use crate::{
    domain::auth::{
        models::{RegisterUserError, RegisterUserRequest},
        ports::AuthService,
    },
    inbound::http::AppState,
};

use axum::{extract::State, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub identifier: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user_id: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RegisterUserHttpRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

impl RegisterUserHttpRequest {
    fn try_into_domain(self) -> Result<RegisterUserRequest, RegisterUserError> {
        Ok(
            RegisterUserRequest::new(&self.username, &self.email, &self.password)
                .map_err(|e| RegisterUserError::InvalidRequest(e))?,
        )
    }
}

pub async fn authenticate_user<AS: AuthService>(
    State(state): State<AppState<AS>>,
    identifier: &str,
    password: &str,
) -> Result<LoginResponse, StatusCode> {
    todo!()
}

pub async fn login<AS: AuthService>(
    State(app_state): State<AppState<AS>>,
    Json(LoginRequest {
        identifier,
        password,
    }): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    todo!()
}

pub async fn register_user<AS: AuthService>(
    app_state: State<AppState<AS>>,
    username: &str,
    email: &str,
    password: &str,
) -> Result<LoginResponse, StatusCode> {
    todo!()
}

pub async fn register<AS: AuthService>(
    State(app_state): State<AppState<AS>>,
    Json(register_user_request): Json<RegisterUserRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    todo!()
}
