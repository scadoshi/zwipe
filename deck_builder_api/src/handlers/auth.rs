// External crate imports
use axum::{extract::State, http::StatusCode, response::Json};
use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool},
    PgConnection, RunQueryDsl,
};
use tracing::{error, warn};

// define DbPool from the more complex type
type DbPool = Pool<ConnectionManager<PgConnection>>;

// Internal imports
use crate::{
    auth::{jwt::generate_jwt, password::verify_password},
    models::{
        login::{LoginRequest, LoginResponse},
        user,
    },
    schema::users,
};

pub async fn authenticate_user(
    pool: DbPool,
    identifier: &str,
    password: &str,
) -> Result<LoginResponse, StatusCode> {
    let mut conn = pool.get().map_err(|e| {
        error!(
            "Failed to get connection to database connection pool with error: {:?}",
            e
        );
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

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
    Json(login_request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let (identifier, password) = (login_request.identifier, login_request.password);
    authenticate_user(pool, identifier.as_str(), password.as_str())
        .await
        .map(|result| Json(result))
}
