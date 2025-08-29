use crate::domain::{
    auth::{
        models::{
            AuthenticateUserError, AuthenticateUserRequest, AuthenticateUserRequestError,
            AuthenticateUserSuccessResponse, ChangePasswordError, ChangePasswordRequest,
            ChangePasswordRequestError, RegisterUserError, RegisterUserRequest,
            RegisterUserRequestError,
        },
        ports::AuthService,
    },
    card::ports::CardService,
    health::ports::HealthService,
    user::ports::UserService,
};
use crate::inbound::http::{ApiError, ApiSuccess, AppState};
use anyhow::anyhow;
use axum::{extract::State, http::StatusCode, Json};
use serde::Deserialize;

// ==========
//  register
// ==========

impl From<RegisterUserError> for ApiError {
    fn from(value: RegisterUserError) -> Self {
        match value {
            RegisterUserError::Duplicate => Self::UnprocessableEntity(
                "user with that username or email already exists".to_string(),
            ),

            RegisterUserError::InvalidRequest(RegisterUserRequestError::InvalidUsername(e)) => {
                Self::UnprocessableEntity(format!(
                    "invalid request: {}",
                    RegisterUserRequestError::InvalidUsername(e)
                ))
            }

            RegisterUserError::InvalidRequest(RegisterUserRequestError::InvalidEmail(e)) => {
                Self::UnprocessableEntity(format!(
                    "invalid request: {}",
                    RegisterUserRequestError::InvalidEmail(e)
                ))
            }

            e => {
                tracing::error!("{:?}\n{}", e, anyhow!("{e}").backtrace());
                Self::InternalServerError("Internal server error".to_string())
            }
        }
    }
}

impl From<RegisterUserRequestError> for ApiError {
    fn from(value: RegisterUserRequestError) -> Self {
        match value {
            RegisterUserRequestError::InvalidUsername(e) => {
                Self::UnprocessableEntity(format!("invalid username: {}", e))
            }
            RegisterUserRequestError::InvalidEmail(e) => {
                Self::UnprocessableEntity(format!("invalid email: {}", e))
            }
            RegisterUserRequestError::InvalidPassword(e) => {
                Self::UnprocessableEntity(format!("invalid password {}", e))
            }
            e => {
                tracing::error!("{:?}\n{}", e, anyhow!("{e}").backtrace());
                Self::InternalServerError("Internal server error".to_string())
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RegisterUserRequestBody {
    username: String,
    email: String,
    password: String,
}

impl TryFrom<RegisterUserRequestBody> for RegisterUserRequest {
    type Error = RegisterUserRequestError;
    fn try_from(value: RegisterUserRequestBody) -> Result<Self, Self::Error> {
        RegisterUserRequest::new(&value.username, &value.email, &value.password)
    }
}

pub async fn register_user<AS, US, HS, CS>(
    State(state): State<AppState<AS, US, HS, CS>>,
    Json(body): Json<RegisterUserRequestBody>,
) -> Result<ApiSuccess<AuthenticateUserSuccessResponse>, ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
{
    let req = RegisterUserRequest::new(&body.username, &body.email, &body.password)?;

    state
        .auth_service
        .register_user(&req)
        .await
        .map_err(ApiError::from)
        .map(|response| ApiSuccess::new(StatusCode::CREATED, response.into()))
}

// ==============
//  authenticate
// ==============

impl From<AuthenticateUserError> for ApiError {
    fn from(value: AuthenticateUserError) -> Self {
        match value {
            AuthenticateUserError::UserNotFound => {
                Self::Unauthorized("invalid credentials".to_string())
            }

            AuthenticateUserError::InvalidPassword => {
                Self::Unauthorized("invalid credentials".to_string())
            }

            e => {
                tracing::error!("{:?}\n{}", e, anyhow!("{e}").backtrace());
                Self::InternalServerError("Internal server error".to_string())
            }
        }
    }
}

impl From<AuthenticateUserRequestError> for ApiError {
    fn from(value: AuthenticateUserRequestError) -> Self {
        match value {
            AuthenticateUserRequestError::MissingIdentifier => {
                Self::UnprocessableEntity("username or email must be present".to_string())
            }
            AuthenticateUserRequestError::MissingPassword => {
                Self::UnprocessableEntity("password must be present".to_string())
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AuthenticateUserRequestBody {
    identifier: String,
    password: String,
}

impl TryFrom<AuthenticateUserRequestBody> for AuthenticateUserRequest {
    type Error = AuthenticateUserRequestError;
    fn try_from(value: AuthenticateUserRequestBody) -> Result<Self, Self::Error> {
        AuthenticateUserRequest::new(&value.identifier, &value.password)
    }
}

pub async fn authenticate_user<AS, US, HS, CS>(
    State(state): State<AppState<AS, US, HS, CS>>,
    Json(body): Json<AuthenticateUserRequestBody>,
) -> Result<ApiSuccess<AuthenticateUserSuccessResponse>, ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
{
    let req = AuthenticateUserRequest::new(&body.identifier, &body.password)?;

    state
        .auth_service
        .authenticate_user(&req)
        .await
        .map_err(ApiError::from)
        .map(|response| ApiSuccess::new(StatusCode::OK, response.into()))
}

// =================
//  change password
// =================

impl From<ChangePasswordError> for ApiError {
    fn from(value: ChangePasswordError) -> Self {
        match value {
            ChangePasswordError::UserNotFound => {
                Self::UnprocessableEntity("user not found".to_string())
            }
            e => {
                tracing::error!("{:?}\n{}", e, anyhow!("{e}").backtrace());
                Self::InternalServerError("Internal server error".to_string())
            }
        }
    }
}

impl From<ChangePasswordRequestError> for ApiError {
    fn from(value: ChangePasswordRequestError) -> Self {
        match value {
            ChangePasswordRequestError::InvalidId(e) => {
                Self::UnprocessableEntity(format!("invalid id {}", e))
            }
            ChangePasswordRequestError::InvalidPassword(e) => {
                Self::UnprocessableEntity(format!("invalid password {}", e))
            }
            e => {
                tracing::error!("{:?}\n{}", e, anyhow!("{e}").backtrace());
                Self::InternalServerError("Internal server error".to_string())
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequestBody {
    id: String,
    new_password: String,
}

impl TryFrom<ChangePasswordRequestBody> for ChangePasswordRequest {
    type Error = ChangePasswordRequestError;
    fn try_from(value: ChangePasswordRequestBody) -> Result<Self, Self::Error> {
        ChangePasswordRequest::new(&value.id, &value.new_password)
    }
}

pub async fn change_password<AS, US, HS, CS>(
    State(state): State<AppState<AS, US, HS, CS>>,
    Json(body): Json<ChangePasswordRequestBody>,
) -> Result<ApiSuccess<()>, ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
{
    let req = ChangePasswordRequest::new(&body.id, &body.new_password)?;

    state
        .auth_service
        .change_password(&req)
        .await
        .map_err(ApiError::from)
        .map(|_| ApiSuccess::new(StatusCode::OK, ()))
}
