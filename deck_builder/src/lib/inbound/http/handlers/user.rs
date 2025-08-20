use anyhow::anyhow;
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::domain::user::models::{
    CreateUserError, CreateUserRequest, CreateUserRequestError, DeleteUserError, GetUserError,
    UpdateUserError, User,
};

// =================================================
//                   Errors
// =================================================

#[derive(Debug)]
pub enum ApiError {
    InternalServerError(String),
    InvalidRequest(String),
    NotFound(String),
}

impl From<anyhow::Error> for ApiError {
    fn from(e: anyhow::Error) -> Self {
        Self::InternalServerError(e.to_string())
    }
}

impl From<CreateUserError> for ApiError {
    fn from(e: CreateUserError) -> Self {
        match e {
            CreateUserError::Duplicate => {
                Self::InvalidRequest("User with that username or email already exists".to_string())
            }

            CreateUserError::InvalidRequest(CreateUserRequestError::InvalidUsername(e)) => {
                Self::InvalidRequest(format!(
                    "Invalid request: {}",
                    CreateUserRequestError::InvalidUsername(e)
                ))
            }

            CreateUserError::InvalidRequest(CreateUserRequestError::InvalidEmail(e)) => {
                Self::InvalidRequest(format!(
                    "Invalid email: {}",
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

impl From<GetUserError> for ApiError {
    fn from(e: GetUserError) -> Self {
        match e {
            GetUserError::NotFound => Self::NotFound("User not found".to_string()),

            e => {
                tracing::error!("{:?}\n{}", e, anyhow!("{e}").backtrace());
                Self::InternalServerError("Internal server error".to_string())
            }
        }
    }
}

impl From<UpdateUserError> for ApiError {
    fn from(e: UpdateUserError) -> Self {
        match e {
            UpdateUserError::Duplicate => {
                Self::InvalidRequest("User with that username or email already exists".to_string())
            }
            UpdateUserError::UserNotFound => Self::NotFound("User not found".to_string()),
            e => {
                tracing::error!("{:?}\n{}", e, anyhow!("{e}").backtrace());
                Self::InternalServerError("Internal server error".to_string())
            }
        }
    }
}

impl From<DeleteUserError> for ApiError {
    fn from(e: DeleteUserError) -> Self {
        match e {
            DeleteUserError::NotFound => Self::NotFound("User not found".to_string()),
            e => {
                tracing::error!("{:?}\n{}", e, anyhow!("{e}").backtrace());
                Self::InternalServerError("Internal server error".to_string())
            }
        }
    }
}

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

            ApiError::InvalidRequest(message) => (
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

// might not need this check later
impl ApiResponseBody<ApiErrorData> {
    pub fn new_error(status_code: StatusCode, message: String) -> Self {
        Self {
            status_code: status_code.as_u16(),
            data: ApiErrorData { message },
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

/// For returning User data from methods
/// Create, Get and Update use this
#[derive(Debug, Serialize)]
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
