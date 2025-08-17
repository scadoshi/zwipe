use crate::{domain::user::{models::{User, UserCreationError, UserCreationRequest}, ports::UserService}, inbound::http::AppState};

use axum::{extract::State, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};
use sqlx::query_as;
use tracing::{error, warn};

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
    let user = query_as!(
        User,
        "SELECT * FROM users WHERE email = $1 OR username = $1",
        identifier
    )
    .fetch_one(&app_state.db_pool)
    .await
    .map_err(|e| {
        warn!("Failed authentication for {:?}. Error: {:?}", identifier, e);
        StatusCode::UNAUTHORIZED
    })?;

    let verified = verify_password(&password, &user.password_hash).map_err(|e| {
        error!("Failed to verify password. Error: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if !verified {
        warn!("Failed login attempt for user_id: {}", user.id);
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = generate_jwt(user.id, user.email, &app_state.jwt_config.secret).map_err(|e| {
        error!("Failed to generate json web token. Error: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(LoginResponse {
        user_id: user.id,
        token,
    })
}

pub async fn login<US: UserService>(
    State(app_state): State<AppState<US>>,
    Json(LoginRequest {
        identifier,
        password,
    }): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    authenticate_user(app_state, identifier.as_str(), password.as_str())
        .await
        .map(|result| Json(result))
}

pub async fn register_user(
    app_state: AppState,
    username: &str,
    email: &str,
    password: &str,
) -> Result<LoginResponse, StatusCode> {
    let password_hash = hash_password(password).map_err(|e| {
        error!("Failed to hash password. Error: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let user = query_as!(
        User,
        "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3) RETURNING *",
        username, email, password_hash
    )
    .fetch_one(&app_state.db_pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(db_e) if db_e.code() == Some("23505".into()) => {
            warn!(
                "Failed registration attempt for (email={:?}, username={:?}). Error: (code={:?}, message={:?})",
                username,
                email,
                db_e.code(),
                db_e.message()
            );
            StatusCode::CONFLICT
        }
        sqlx::Error::Database(db_e) => {
            error!("Failed to create user. Database error: {:?}", db_e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
        e => {
            error!("Failed to create user. Error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

    let token = generate_jwt(user.id, user.email, &app_state.jwt_config.secret).map_err(|e| {
        error!("Failed to generate a json web token. Error: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Jwt::generate(user.id, user.email, )

    Ok(LoginResponse {
        token,
        user_id: user.id,
    })
}

pub async fn register<US: UserService>(
    State(app_state): State<AppState<US>>,
    Json(registration_request): Json<UserCreationRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    register_user(
        app_state,
        &registration_request.username,
        &registration_request.email,
        &registration_request.password,
    )
    .await
    .map(|result| Json(result))
}
