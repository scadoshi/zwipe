use crate::{
    domain::{
        auth::ports::AuthService,
        card::ports::CardService,
        deck::ports::DeckService,
        health::ports::HealthService,
        user::{
            models::{
                CreateUser, CreateUserError, DeleteUser, DeleteUserError, GetUser, GetUserError,
                InvalidCreateUser, InvalidUpdateUser, UpdateUser, UpdateUserError, User,
            },
            ports::UserService,
        },
    },
    inbound::http::{middleware::AuthenticatedUser, ApiError, AppState, Log500},
};
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

            e => e.log_500(),
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

pub async fn create_user<AS, US, HS, CS, DS>(
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Json(body): Json<HttpCreateUser>,
) -> Result<(StatusCode, Json<HttpUser>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let request = CreateUser::new(&body.username, &body.email)?;

    state
        .user_service
        .create_user(&request)
        .await
        .map_err(ApiError::from)
        .map(|user| (StatusCode::CREATED, Json(user.into())))
}

// =====
//  get
// =====

impl From<GetUserError> for ApiError {
    fn from(value: GetUserError) -> Self {
        match value {
            GetUserError::NotFound => Self::NotFound("user not found".to_string()),

            e => e.log_500(),
        }
    }
}

pub async fn get_user<AS, US, HS, CS, DS>(
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Path(identifier): Path<String>,
) -> Result<(StatusCode, Json<HttpUser>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let request = GetUser::new(&identifier);

    state
        .user_service
        .get_user(&request)
        .await
        .map_err(ApiError::from)
        .map(|user| (StatusCode::OK, Json(user.into())))
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
            e => e.log_500(),
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
pub struct HttpUpdateUser {
    id: String,
    username: Option<String>,
    email: Option<String>,
}

impl TryFrom<HttpUpdateUser> for UpdateUser {
    type Error = InvalidUpdateUser;
    fn try_from(value: HttpUpdateUser) -> Result<Self, Self::Error> {
        UpdateUser::new(&value.id, value.username, value.email)
    }
}

pub async fn update_user<AS, US, HS, CS, DS>(
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Json(body): Json<HttpUpdateUser>,
) -> Result<(StatusCode, Json<HttpUser>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let request = UpdateUser::new(&body.id, body.username, body.email)?;

    state
        .user_service
        .update_user(&request)
        .await
        .map_err(ApiError::from)
        .map(|user| (StatusCode::OK, Json(user.into())))
}

// ========
//  delete
// ========

impl From<DeleteUserError> for ApiError {
    fn from(value: DeleteUserError) -> Self {
        match value {
            DeleteUserError::NotFound => Self::NotFound("user not found".to_string()),
            e => e.log_500(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct HttpDeleteUser(String);

impl TryFrom<HttpDeleteUser> for DeleteUser {
    type Error = uuid::Error;
    fn try_from(value: HttpDeleteUser) -> Result<Self, Self::Error> {
        DeleteUser::new(&value.0)
    }
}

pub async fn delete_user<AS, US, HS, CS, DS>(
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Path(request): Path<HttpDeleteUser>,
) -> Result<(StatusCode, Json<()>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let request = DeleteUser::new(&request.0)?;

    state
        .user_service
        .delete_user(&request)
        .await
        .map_err(ApiError::from)
        .map(|_| (StatusCode::OK, Json(())))
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

impl From<User> for HttpUser {
    fn from(user: User) -> Self {
        Self {
            id: user.id.to_string(),
            username: user.username.to_string(),
            email: user.email.to_string(),
        }
    }
}
