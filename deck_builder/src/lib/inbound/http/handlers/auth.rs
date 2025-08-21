// =================================================
//             All api error mappings
// =================================================

use anyhow::anyhow;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{
    domain::{
        auth::{
            models::{
                jwt::{Jwt, JwtSecret},
                AuthenticateUserError, AuthenticateUserRequest, AuthenticateUserRequestError,
                AuthenticateUserSuccessResponse, ChangePasswordError, ChangePasswordRequest,
                ChangePasswordRequestError, RegisterUserError, RegisterUserRequest,
                RegisterUserRequestError,
            },
            ports::AuthService,
        },
        user::{models::User, ports::UserService},
    },
    inbound::http::AppState,
};

#[derive(Debug)]
pub enum ApiError {
    InternalServerError(String),
    Unauthorized(String),
    UnprocessableEntity(String),
}

impl From<anyhow::Error> for ApiError {
    fn from(value: anyhow::Error) -> Self {
        Self::InternalServerError(value.to_string())
    }
}

// =================================================
//                   Create / Register
// =================================================

impl From<RegisterUserError> for ApiError {
    fn from(value: RegisterUserError) -> Self {
        match value {
            RegisterUserError::Duplicate => Self::UnprocessableEntity(
                "User with that username or email already exists".to_string(),
            ),

            RegisterUserError::InvalidRequest(RegisterUserRequestError::InvalidUsername(e)) => {
                Self::UnprocessableEntity(format!(
                    "Invalid request: {}",
                    RegisterUserRequestError::InvalidUsername(e)
                ))
            }

            RegisterUserError::InvalidRequest(RegisterUserRequestError::InvalidEmail(e)) => {
                Self::UnprocessableEntity(format!(
                    "Invalid request: {}",
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
                Self::UnprocessableEntity(format!("Invalid username: {}", e))
            }
            RegisterUserRequestError::InvalidEmail(e) => {
                Self::UnprocessableEntity(format!("Invalid email: {}", e))
            }
            RegisterUserRequestError::InvalidPassword(e) => {
                Self::UnprocessableEntity(format!("Invalid password {}", e))
            }
            e => {
                tracing::error!("{:?}\n{}", e, anyhow!("{e}").backtrace());
                Self::InternalServerError("Internal server error".to_string())
            }
        }
    }
}

// =================================================
//                   Get / Authorize
// =================================================

impl From<AuthenticateUserError> for ApiError {
    fn from(value: AuthenticateUserError) -> Self {
        match value {
            AuthenticateUserError::UserNotFound => {
                Self::Unauthorized("Invalid credentials".to_string())
            }

            AuthenticateUserError::InvalidPassword => {
                Self::Unauthorized("Invalid credentials".to_string())
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
                Self::UnprocessableEntity("Username or email must be present".to_string())
            }
            AuthenticateUserRequestError::MissingPassword => {
                Self::UnprocessableEntity("Password must be present".to_string())
            }
        }
    }
}

// =================================================
//            Update / Change password
// =================================================

impl From<ChangePasswordError> for ApiError {
    fn from(value: ChangePasswordError) -> Self {
        match value {
            ChangePasswordError::UserNotFound => {
                Self::UnprocessableEntity("User not found".to_string())
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
                Self::UnprocessableEntity(format!("Invalid ID {}", e))
            }
            ChangePasswordRequestError::InvalidPassword(e) => {
                Self::UnprocessableEntity(format!("Invalid password {}", e))
            }
            e => {
                tracing::error!("{:?}\n{}", e, anyhow!("{e}").backtrace());
                Self::InternalServerError("Internal server error".to_string())
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

            ApiError::Unauthorized(message) => (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponseBody::new_error(
                    StatusCode::UNAUTHORIZED,
                    message,
                )),
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

// =================================================
//                   Response
// =================================================

/// For returning User data from methods
/// Register uses this
#[derive(Debug, Serialize, PartialEq)]
pub struct AuthenticateUserSuccessResponseData {
    user: User,
    token: Jwt,
    expires_at: usize,
}

impl From<AuthenticateUserSuccessResponse> for AuthenticateUserSuccessResponseData {
    fn from(value: AuthenticateUserSuccessResponse) -> Self {
        Self {
            user: value.user,
            token: value.token,
            expires_at: value.expires_at,
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

pub async fn register_user<AS: AuthService, US: UserService>(
    State(state): State<AppState<AS, US>>,
    Json(body): Json<RegisterUserRequestBody>,
    jwt_secret: JwtSecret,
) -> Result<ApiSuccess<AuthenticateUserSuccessResponseData>, ApiError> {
    let req = RegisterUserRequest::new(&body.username, &body.email, &body.password)?;

    state
        .auth_service
        .register_user(&req, jwt_secret)
        .await
        .map_err(ApiError::from)
        .map(|response| ApiSuccess::new(StatusCode::CREATED, response.into()))
}

pub async fn authenticate_user<AS: AuthService, US: UserService>(
    State(state): State<AppState<AS, US>>,
    Json(body): Json<AuthenticateUserRequestBody>,
    jwt_secret: JwtSecret,
) -> Result<ApiSuccess<AuthenticateUserSuccessResponseData>, ApiError> {
    let req = AuthenticateUserRequest::new(&body.identifier, &body.password)?;

    state
        .auth_service
        .authenticate_user(&req, jwt_secret)
        .await
        .map_err(ApiError::from)
        .map(|response| ApiSuccess::new(StatusCode::OK, response.into()))
}

pub async fn change_password<AS: AuthService, US: UserService>(
    State(state): State<AppState<AS, US>>,
    Json(body): Json<ChangePasswordRequestBody>,
) -> Result<ApiSuccess<()>, ApiError> {
    let req = ChangePasswordRequest::new(&body.id, &body.new_password)?;

    state
        .auth_service
        .change_password(&req)
        .await
        .map_err(ApiError::from)
        .map(|_| ApiSuccess::new(StatusCode::OK, ()))
}
