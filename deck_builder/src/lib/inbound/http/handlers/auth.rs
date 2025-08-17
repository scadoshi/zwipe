use crate::{domain::user::{models::{User, UserCreationRequest}, ports::UserService}, inbound::http::AppState};

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
pub struct UserCreationHttpRequestBody {
    pub username: String,
    pub email: String,
    pub password: String,
}

impl UserCreationHttpRequestBody {
    fn try_into_domain(self) -> Result<UserCreationRequest, UserCreationError> {
        Ok(UserCreationRequest::new(&self.username, &self.email, &self.password)?)
    }
}

pub async fn authenticate_user<US: UserService>(
    State(state): State<AppState<US>>,
    identifier: &str,
    password: &str,
) -> Result<LoginResponse, StatusCode> {
    todo()!
}

pub async fn login<US: UserService>(
    State(app_state): State<AppState<US>>,
    Json(LoginRequest {
        identifier,
        password,
    }): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    todo!()
}

pub async fn register_user(
    app_state: AppState,
    username: &str,
    email: &str,
    password: &str,
) -> Result<LoginResponse, StatusCode> {
    todo!()
}

pub async fn register<US: UserService>(
    State(app_state): State<AppState<US>>,
    Json(registration_request): Json<UserCreationRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    todo!()
}
