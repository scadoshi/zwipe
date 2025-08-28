// internal
use crate::{
    domain::{
        auth::ports::AuthService,
        card::ports::CardService,
        health::ports::HealthService,
        user::{
            models::{
                CreateUserError, CreateUserRequest, CreateUserRequestError, DeleteUserError,
                DeleteUserRequest, DeleteUserRequestError, GetUserError, GetUserRequest,
                UpdateUserError, UpdateUserRequest, UpdateUserRequestError, User,
            },
            ports::UserService,
        },
    },
    inbound::http::{ApiError, ApiSuccess, AppState},
};
// external
use anyhow::anyhow;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

// ========
//  create
// ========

impl From<CreateUserError> for ApiError {
    fn from(value: CreateUserError) -> Self {
        match value {
            CreateUserError::Duplicate => Self::UnprocessableEntity(
                "user with that username or email already exists".to_string(),
            ),

            CreateUserError::InvalidRequest(CreateUserRequestError::InvalidUsername(e)) => {
                Self::UnprocessableEntity(format!(
                    "invalid request: {}",
                    CreateUserRequestError::InvalidUsername(e)
                ))
            }

            CreateUserError::InvalidRequest(CreateUserRequestError::InvalidEmail(e)) => {
                Self::UnprocessableEntity(format!(
                    "invalid request: {}",
                    CreateUserRequestError::InvalidEmail(e)
                ))
            }

            e => {
                tracing::error!("{:?}\n{}", e, anyhow!("{e}").backtrace());
                Self::InternalServerError("internal server error".to_string())
            }
        }
    }
}

impl From<CreateUserRequestError> for ApiError {
    fn from(value: CreateUserRequestError) -> Self {
        match value {
            CreateUserRequestError::InvalidUsername(e) => {
                Self::UnprocessableEntity(format!("invalid username: {}", e))
            }
            CreateUserRequestError::InvalidEmail(e) => {
                Self::UnprocessableEntity(format!("invalid email: {}", e))
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequestBody {
    username: String,
    email: String,
}

impl TryFrom<CreateUserRequestBody> for CreateUserRequest {
    type Error = CreateUserRequestError;
    fn try_from(value: CreateUserRequestBody) -> Result<Self, Self::Error> {
        CreateUserRequest::new(&value.username, &value.email)
    }
}

pub async fn create_user<AS, US, HS, CS>(
    State(state): State<AppState<AS, US, HS, CS>>,
    Json(body): Json<CreateUserRequestBody>,
) -> Result<ApiSuccess<UserResponseData>, ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
{
    let req = CreateUserRequest::new(&body.username, &body.email)?;

    state
        .user_service
        .create_user(&req)
        .await
        .map_err(ApiError::from)
        .map(|ref user| ApiSuccess::new(StatusCode::CREATED, user.into()))
}

// =====
//  get
// =====

impl From<GetUserError> for ApiError {
    fn from(value: GetUserError) -> Self {
        match value {
            GetUserError::NotFound => Self::NotFound("user not found".to_string()),

            e => {
                tracing::error!("{:?}\n{}", e, anyhow!("{e}").backtrace());
                Self::InternalServerError("internal server error".to_string())
            }
        }
    }
}

pub async fn get_user<AS, US, HS, CS>(
    State(state): State<AppState<AS, US, HS, CS>>,
    Path(identifier): Path<String>,
) -> Result<ApiSuccess<UserResponseData>, ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
{
    let req = GetUserRequest::new(&identifier);

    state
        .user_service
        .get_user(&req)
        .await
        .map_err(ApiError::from)
        .map(|ref user| ApiSuccess::new(StatusCode::OK, user.into()))
}

// ========
//  update
// ========

impl From<UpdateUserError> for ApiError {
    fn from(value: UpdateUserError) -> Self {
        match value {
            UpdateUserError::Duplicate => Self::UnprocessableEntity(
                "user with that username or email already exists".to_string(),
            ),
            UpdateUserError::UserNotFound => Self::NotFound("user not found".to_string()),
            e => {
                tracing::error!("{:?}\n{}", e, anyhow!("{e}").backtrace());
                Self::InternalServerError("internal server error".to_string())
            }
        }
    }
}

impl From<UpdateUserRequestError> for ApiError {
    fn from(value: UpdateUserRequestError) -> Self {
        match value {
            UpdateUserRequestError::InvalidId(e) => {
                Self::UnprocessableEntity(format!("invalid ID {}", e))
            }
            UpdateUserRequestError::InvalidUsername(e) => {
                Self::UnprocessableEntity(format!("invalid username {}", e))
            }
            UpdateUserRequestError::InvalidEmail(e) => {
                Self::UnprocessableEntity(format!("invalid email {}", e))
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequestBody {
    id: String,
    username: Option<String>,
    email: Option<String>,
}

impl TryFrom<UpdateUserRequestBody> for UpdateUserRequest {
    type Error = UpdateUserRequestError;
    fn try_from(value: UpdateUserRequestBody) -> Result<Self, Self::Error> {
        UpdateUserRequest::new(&value.id, value.username, value.email)
    }
}

pub async fn update_user<AS, US, HS, CS>(
    State(state): State<AppState<AS, US, HS, CS>>,
    Json(body): Json<UpdateUserRequestBody>,
) -> Result<ApiSuccess<UserResponseData>, ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
{
    let req = UpdateUserRequest::new(&body.id, body.username, body.email)?;

    state
        .user_service
        .update_user(&req)
        .await
        .map_err(ApiError::from)
        .map(|ref user| ApiSuccess::new(StatusCode::OK, user.into()))
}

// ========
//  delete
// ========

impl From<DeleteUserError> for ApiError {
    fn from(value: DeleteUserError) -> Self {
        match value {
            DeleteUserError::NotFound => Self::NotFound("user not found".to_string()),
            e => {
                tracing::error!("{:?}\n{}", e, anyhow!("{e}").backtrace());
                Self::InternalServerError("internal server error".to_string())
            }
        }
    }
}

impl From<DeleteUserRequestError> for ApiError {
    fn from(value: DeleteUserRequestError) -> Self {
        match value {
            DeleteUserRequestError::MissingId => {
                Self::UnprocessableEntity("id must be present".to_string())
            }
            DeleteUserRequestError::FailedUuid(e) => {
                Self::UnprocessableEntity(format!("failed to parse Uuid: {}", e))
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct DeleteUserRequestBody {
    id: String,
}

impl TryFrom<DeleteUserRequestBody> for DeleteUserRequest {
    type Error = DeleteUserRequestError;
    fn try_from(value: DeleteUserRequestBody) -> Result<Self, Self::Error> {
        DeleteUserRequest::new(&value.id)
    }
}

pub async fn delete_user<AS, US, HS, CS>(
    State(state): State<AppState<AS, US, HS, CS>>,
    Json(body): Json<DeleteUserRequestBody>,
) -> Result<ApiSuccess<()>, ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
{
    let req = DeleteUserRequest::new(&body.id)?;

    state
        .user_service
        .delete_user(&req)
        .await
        .map_err(ApiError::from)
        .map(|_| ApiSuccess::new(StatusCode::OK, ()))
}

// ==========
//  response
// ==========

/// for returning `User` data from methods
///
/// create, get and update use this
#[derive(Debug, Serialize, PartialEq)]
pub struct UserResponseData {
    id: String,
    username: String,
    email: String,
}

impl From<&User> for UserResponseData {
    fn from(user: &User) -> Self {
        Self {
            id: user.id.to_string(),
            username: user.username.to_string(),
            email: user.email.to_string(),
        }
    }
}
