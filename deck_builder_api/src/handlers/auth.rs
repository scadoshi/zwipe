// External
use axum::{extract::State, http::StatusCode, response::Json};
use diesel::{
    dsl::insert_into,
    prelude::*,
    result::{DatabaseErrorKind, Error::DatabaseError},
    RunQueryDsl,
};
use serde::{Deserialize, Serialize};
use tracing::{error, warn};

// Internal
use crate::{
    auth::{
        jwt::generate_jwt,
        password::{hash_password, verify_password},
    },
    models::user,
    schema::users,
    utils::connect_to,
    AppState,
};

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

#[derive(Deserialize)]
pub struct RegistrationRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

pub async fn authenticate_user(
    app_state: AppState,
    identifier: &str,
    password: &str,
) -> Result<LoginResponse, StatusCode> {
    let mut conn = connect_to(app_state.db_pool)?;

    let user = users::table
        .filter(
            users::email
                .eq(&identifier)
                .or(users::username.eq(&identifier)),
        )
        .first::<user::User>(&mut conn)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

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

pub async fn login(
    State(app_state): State<AppState>,
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
    let mut conn = connect_to(app_state.db_pool)?;

    let password_hash = hash_password(password).map_err(|e| {
        error!("Failed to hash password. Error: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let new_user = user::NewUser {
        email: email.to_string(),
        username: username.to_string(),
        password_hash: password_hash.to_string(),
    };

    let user = insert_into(users::table)
        .values(&new_user)
        .get_result::<user::User>(&mut conn)
        .map_err(|e| match e {
            DatabaseError(DatabaseErrorKind::UniqueViolation, database_error_information) => {
                warn!(
                    "Failed registration attempt (email={:?}, username={:?}). Error: ({:?}, {:?})",
                    new_user.username,
                    new_user.email,
                    DatabaseErrorKind::UniqueViolation,
                    database_error_information
                );
                StatusCode::CONFLICT
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

    Ok(LoginResponse {
        token,
        user_id: user.id,
    })
}

pub async fn register(
    State(app_state): State<AppState>,
    Json(registration_request): Json<RegistrationRequest>,
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
