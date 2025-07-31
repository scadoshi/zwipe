// External
use axum::{extract::State, http::StatusCode, response::Json};
use diesel::{
    dsl::insert_into, prelude::*, r2d2::{ConnectionManager, Pool}, result::{DatabaseErrorKind, Error::DatabaseError}, PgConnection, RunQueryDsl
};
use serde::{Deserialize, Serialize};
use tracing::{error, warn};

// Internal
use crate::{
    auth::{jwt::generate_jwt, password::{hash_password, verify_password}},
    models::user,
    schema::users,
    utils::connect_to,
};

// define DbPool from the more complex type
type DbPool = Pool<ConnectionManager<PgConnection>>;

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
    pool: DbPool,
    identifier: &str,
    password: &str,
) -> Result<LoginResponse, StatusCode> {
    let mut conn = connect_to(pool)?;

    let user = users::table
        .filter(
            users::email
                .eq(&identifier)
                .or(users::username.eq(&identifier)),
        )
        .first::<user::User>(&mut conn)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let verified = verify_password(&password, &user.password_hash).map_err(|e| {
        error!("Failed to verify password with error: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if !verified {
        warn!("Failed login attempt for user_id: {}", user.id);
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = generate_jwt(user.id, user.email).map_err(|e| {
        error!("Failed to generate json web token with error: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(LoginResponse {
        user_id: user.id,
        token,
    })
}

pub async fn login(
    State(pool): State<DbPool>,
    Json(LoginRequest { identifier, password }): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    authenticate_user(pool, identifier.as_str(), password.as_str())
        .await
        .map(|result| Json(result))
}

pub async fn register_user(pool: DbPool, username: &str, email: &str, password: &str) -> Result<LoginResponse, StatusCode> {
    let mut conn = connect_to(pool)?;

    let password_hash = hash_password(password).map_err(|e| {
        error!("Failed to hash password with error: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let new_user = user::NewUser {email: email.to_string(), username: username.to_string(), password_hash: password_hash.to_string()};

    let user = insert_into(users::table)
        .values(&new_user)
        .get_result::<user::User>(&mut conn).map_err(|e|{
            match e {
                    DatabaseError(
                        DatabaseErrorKind::UniqueViolation, database_error_information
                    ) => {
                    warn!(
                        "Failed to create user with email={:?} and username={:?} as a user already exists with one of those. Error was: ({:?}, {:?})", 
                        new_user.username, new_user.email, 
                        DatabaseErrorKind::UniqueViolation, 
                        database_error_information
                    );
                    StatusCode::CONFLICT
                }
                e => {
                    error!("Failed to create user with error: {:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            }
        })?;

    let token = generate_jwt(user.id, user.email).map_err(|e| {
        error!("Failed to generate a json web token with error: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(LoginResponse { token, user_id: user.id })
}

pub async fn register(State(pool): State<DbPool>, 
Json(registration_request): Json<RegistrationRequest>) -> Result<Json<LoginResponse>, StatusCode> {
    register_user(pool, &registration_request.username, &registration_request.email, &registration_request.password)
        .await
        .map(|result| Json(result))
}
