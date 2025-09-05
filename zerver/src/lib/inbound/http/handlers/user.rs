use crate::{
    domain::{
        auth::ports::AuthService,
        card::ports::CardService,
        health::ports::HealthService,
        user::{
            models::{
                CreateUser, CreateUserError, DeleteUser, DeleteUserError, GetUser, GetUserError,
                InvalidCreateUser, InvalidUpdateUser, UpdateUser, UpdateUserError, User,
            },
            ports::UserService,
        },
    },
    inbound::http::{ApiError, ApiSuccess, AppState},
};
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

            e => {
                tracing::error!("{:?}\n{}", e, anyhow!("{e}").backtrace());
                Self::InternalServerError("internal server error".to_string())
            }
        }
    }
}

impl From<InvalidCreateUser> for ApiError {
    fn from(value: InvalidCreateUser) -> Self {
        match value {
            InvalidCreateUser::Username(e) => {
                Self::UnprocessableEntity(format!("invalid username: {}", e))
            }
            InvalidCreateUser::Email(e) => {
                Self::UnprocessableEntity(format!("invalid email: {}", e))
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct HttpCreateUser {
    username: String,
    email: String,
}

impl TryFrom<HttpCreateUser> for CreateUser {
    type Error = InvalidCreateUser;
    fn try_from(value: HttpCreateUser) -> Result<Self, Self::Error> {
        CreateUser::new(&value.username, &value.email)
    }
}

pub async fn create_user<AS, US, HS, CS>(
    State(state): State<AppState<AS, US, HS, CS>>,
    Json(body): Json<HttpCreateUser>,
) -> Result<ApiSuccess<HttpUser>, ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
{
    let req = CreateUser::new(&body.username, &body.email)?;

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
) -> Result<ApiSuccess<HttpUser>, ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
{
    let req = GetUser::new(&identifier);

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
            UpdateUserError::NotFound => Self::NotFound("user not found".to_string()),
            e => {
                tracing::error!("{:?}\n{}", e, anyhow!("{e}").backtrace());
                Self::InternalServerError("internal server error".to_string())
            }
        }
    }
}

impl From<InvalidUpdateUser> for ApiError {
    fn from(value: InvalidUpdateUser) -> Self {
        match value {
            InvalidUpdateUser::Id(e) => Self::UnprocessableEntity(format!("invalid ID {}", e)),
            InvalidUpdateUser::Username(e) => {
                Self::UnprocessableEntity(format!("invalid username {}", e))
            }
            InvalidUpdateUser::Email(e) => {
                Self::UnprocessableEntity(format!("invalid email {}", e))
            }
            InvalidUpdateUser::NoUpdates => {
                Self::UnprocessableEntity("must update at least one field".to_string())
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserBody {
    id: String,
    username: Option<String>,
    email: Option<String>,
}

impl TryFrom<UpdateUserBody> for UpdateUser {
    type Error = InvalidUpdateUser;
    fn try_from(value: UpdateUserBody) -> Result<Self, Self::Error> {
        UpdateUser::new(&value.id, value.username, value.email)
    }
}

pub async fn update_user<AS, US, HS, CS>(
    State(state): State<AppState<AS, US, HS, CS>>,
    Json(body): Json<UpdateUserBody>,
) -> Result<ApiSuccess<HttpUser>, ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
{
    let req = UpdateUser::new(&body.id, body.username, body.email)?;

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

#[derive(Debug, Deserialize)]
pub struct DeleteUserBody {
    id: String,
}

impl TryFrom<DeleteUserBody> for DeleteUser {
    type Error = uuid::Error;
    fn try_from(value: DeleteUserBody) -> Result<Self, Self::Error> {
        DeleteUser::new(&value.id)
    }
}

pub async fn delete_user<AS, US, HS, CS>(
    State(state): State<AppState<AS, US, HS, CS>>,
    Json(body): Json<DeleteUserBody>,
) -> Result<ApiSuccess<()>, ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
{
    let req = DeleteUser::new(&body.id)?;

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
pub struct HttpUser {
    id: String,
    username: String,
    email: String,
}

impl From<&User> for HttpUser {
    fn from(user: &User) -> Self {
        Self {
            id: user.id.to_string(),
            username: user.username.to_string(),
            email: user.email.to_string(),
        }
    }
}
