use anyhow::anyhow;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{
    domain::{
        auth::ports::AuthService,
        user::{
            models::{
                CreateUserError, CreateUserRequest, CreateUserRequestError, DeleteUserError,
                DeleteUserRequest, DeleteUserRequestError, GetUserError, GetUserRequest,
                UpdateUserError, UpdateUserRequest, UpdateUserRequestError, User,
            },
            ports::UserService,
        },
    },
    inbound::http::AppState,
};

// =================================================
//             All api error mappings
// =================================================

#[derive(Debug)]
pub enum ApiError {
    InternalServerError(String),
    UnprocessableEntity(String),
    NotFound(String),
}

impl From<anyhow::Error> for ApiError {
    fn from(value: anyhow::Error) -> Self {
        Self::InternalServerError(value.to_string())
    }
}

// =================================================
//                   Create
// =================================================

impl From<CreateUserError> for ApiError {
    fn from(value: CreateUserError) -> Self {
        match value {
            CreateUserError::Duplicate => Self::UnprocessableEntity(
                "User with that username or email already exists".to_string(),
            ),

            CreateUserError::InvalidRequest(CreateUserRequestError::InvalidUsername(e)) => {
                Self::UnprocessableEntity(format!(
                    "Invalid request: {}",
                    CreateUserRequestError::InvalidUsername(e)
                ))
            }

            CreateUserError::InvalidRequest(CreateUserRequestError::InvalidEmail(e)) => {
                Self::UnprocessableEntity(format!(
                    "Invalid request: {}",
                    CreateUserRequestError::InvalidEmail(e)
                ))
            }

            e => {
                tracing::error!("{:?}\n{}", e, anyhow!("{e}").backtrace());
                Self::InternalServerError("Internal server error".to_string())
            }
        }
    }
}

impl From<CreateUserRequestError> for ApiError {
    fn from(value: CreateUserRequestError) -> Self {
        match value {
            CreateUserRequestError::InvalidUsername(e) => {
                Self::UnprocessableEntity(format!("Invalid username: {}", e))
            }
            CreateUserRequestError::InvalidEmail(e) => {
                Self::UnprocessableEntity(format!("Invalid email: {}", e))
            }
        }
    }
}

// =================================================
//                   Get
// =================================================

impl From<GetUserError> for ApiError {
    fn from(value: GetUserError) -> Self {
        match value {
            GetUserError::NotFound => Self::NotFound("User not found".to_string()),

            e => {
                tracing::error!("{:?}\n{}", e, anyhow!("{e}").backtrace());
                Self::InternalServerError("Internal server error".to_string())
            }
        }
    }
}

//
// since building the request is infallible we don't handle that
//

// =================================================
//                   Update
// =================================================

impl From<UpdateUserError> for ApiError {
    fn from(value: UpdateUserError) -> Self {
        match value {
            UpdateUserError::Duplicate => Self::UnprocessableEntity(
                "User with that username or email already exists".to_string(),
            ),
            UpdateUserError::UserNotFound => Self::NotFound("User not found".to_string()),
            e => {
                tracing::error!("{:?}\n{}", e, anyhow!("{e}").backtrace());
                Self::InternalServerError("Internal server error".to_string())
            }
        }
    }
}

impl From<UpdateUserRequestError> for ApiError {
    fn from(value: UpdateUserRequestError) -> Self {
        match value {
            UpdateUserRequestError::InvalidId(e) => {
                Self::UnprocessableEntity(format!("Invalid ID {}", e))
            }
            UpdateUserRequestError::InvalidUsername(e) => {
                Self::UnprocessableEntity(format!("Invalid username {}", e))
            }
            UpdateUserRequestError::InvalidEmail(e) => {
                Self::UnprocessableEntity(format!("Invalid email {}", e))
            }
        }
    }
}

// =================================================
//                   Delete
// =================================================

impl From<DeleteUserError> for ApiError {
    fn from(value: DeleteUserError) -> Self {
        match value {
            DeleteUserError::NotFound => Self::NotFound("User not found".to_string()),
            e => {
                tracing::error!("{:?}\n{}", e, anyhow!("{e}").backtrace());
                Self::InternalServerError("Internal server error".to_string())
            }
        }
    }
}

impl From<DeleteUserRequestError> for ApiError {
    fn from(value: DeleteUserRequestError) -> Self {
        match value {
            DeleteUserRequestError::MissingId => {
                Self::UnprocessableEntity("ID must be present".to_string())
            }
            DeleteUserRequestError::FailedUuid(e) => {
                Self::UnprocessableEntity(format!("Failed to parse Uuid: {}", e))
            }
        }
    }
}

// =================================================
//     Turning api error into an actual response
// =================================================

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ApiError::InternalServerError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponseBody::new_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )),
            )
                .into_response(),

            ApiError::UnprocessableEntity(message) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(ApiResponseBody::new_error(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    message,
                )),
            )
                .into_response(),

            ApiError::NotFound(message) => (
                StatusCode::NOT_FOUND,
                Json(ApiResponseBody::new_error(StatusCode::NOT_FOUND, message)),
            )
                .into_response(),
        }
    }
}

// =================================================
//                    Parts
// =================================================

// =================================================
//                 Http things
// =================================================

#[derive(Debug, Serialize, PartialEq)]
pub struct ApiErrorData {
    pub message: String,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct ApiResponseBody<T: Serialize + PartialEq> {
    status_code: u16,
    data: T,
}

impl<T: Serialize + PartialEq> ApiResponseBody<T> {
    fn new(status_code: StatusCode, data: T) -> Self {
        ApiResponseBody {
            status_code: status_code.as_u16(),
            data,
        }
    }
}

impl ApiResponseBody<ApiErrorData> {
    pub fn new_error(status_code: StatusCode, message: String) -> Self {
        Self {
            status_code: status_code.as_u16(),
            data: ApiErrorData { message },
        }
    }
}

// =================================================
//                 Request bodies
// =================================================

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

#[derive(Debug, Deserialize)]
pub struct GetUserRequestBody {
    identifier: String,
}

impl From<GetUserRequestBody> for GetUserRequest {
    fn from(value: GetUserRequestBody) -> Self {
        GetUserRequest::new(&value.identifier)
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

// =================================================
//                   Response
// =================================================

/// For returning User data from methods
/// Create, Get and Update use this
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

// =================================================
//                   ApiSuccess
// =================================================

#[derive(Debug)]
pub struct ApiSuccess<T: Serialize + PartialEq>(StatusCode, Json<ApiResponseBody<T>>);

impl<T: Serialize + PartialEq> PartialEq for ApiSuccess<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 .0 == other.1 .0
    }
}

impl<T: Serialize + PartialEq> ApiSuccess<T> {
    fn new(status: StatusCode, data: T) -> Self {
        ApiSuccess(status, Json(ApiResponseBody::new(status, data)))
    }
}

impl<T: Serialize + PartialEq> IntoResponse for ApiSuccess<T> {
    fn into_response(self) -> axum::response::Response {
        (self.0, self.1).into_response()
    }
}

// =================================================
//                   Functions
// =================================================

pub async fn create_user<AS: AuthService, US: UserService>(
    State(state): State<AppState<AS, US>>,
    Json(body): Json<CreateUserRequestBody>,
) -> Result<ApiSuccess<UserResponseData>, ApiError> {
    let req = CreateUserRequest::new(&body.username, &body.email)?;

    state
        .user_service
        .create_user(&req)
        .await
        .map_err(ApiError::from)
        .map(|ref user| ApiSuccess::new(StatusCode::CREATED, user.into()))
}

pub async fn get_user<AS: AuthService, US: UserService>(
    State(state): State<AppState<AS, US>>,
    Json(body): Json<GetUserRequestBody>,
) -> Result<ApiSuccess<UserResponseData>, ApiError> {
    let req = GetUserRequest::new(&body.identifier);

    state
        .user_service
        .get_user(&req)
        .await
        .map_err(ApiError::from)
        .map(|ref user| ApiSuccess::new(StatusCode::OK, user.into()))
}

pub async fn update_user<AS: AuthService, US: UserService>(
    State(state): State<AppState<AS, US>>,
    Json(body): Json<UpdateUserRequestBody>,
) -> Result<ApiSuccess<UserResponseData>, ApiError> {
    let req = UpdateUserRequest::new(&body.id, body.username, body.email)?;

    state
        .user_service
        .update_user(&req)
        .await
        .map_err(ApiError::from)
        .map(|ref user| ApiSuccess::new(StatusCode::OK, user.into()))
}

pub async fn delete_user<AS: AuthService, US: UserService>(
    State(state): State<AppState<AS, US>>,
    Json(body): Json<DeleteUserRequestBody>,
) -> Result<ApiSuccess<()>, ApiError> {
    let req = DeleteUserRequest::new(&body.id)?;

    state
        .user_service
        .delete_user(&req)
        .await
        .map_err(ApiError::from)
        .map(|_| ApiSuccess::new(StatusCode::OK, ()))
}
