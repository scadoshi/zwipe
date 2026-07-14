#[cfg(feature = "zerver")]
use crate::inbound::http::Log500;
#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::{
            models::access_token::InvalidJwt,
            requests::{
                create_session::CreateSessionError,
                enforce_session_maximum::EnforceSessionMaximumError,
                register_user::{InvalidRegisterUser, RegisterUser, RegisterUserError},
            },
        },
        metrics::models::kinds::EventKind,
        user::models::get_user::GetUserError,
    },
    inbound::http::{ApiError, AppState},
};
#[cfg(feature = "zerver")]
use axum::{Json, extract::State, http::StatusCode};
#[cfg(feature = "zerver")]
use zwipe_core::domain::auth::models::session::Session;
#[cfg(feature = "zerver")]
use zwipe_core::http::contracts::auth::HttpRegisterUser;

#[cfg(feature = "zerver")]
impl From<EnforceSessionMaximumError> for ApiError {
    fn from(value: EnforceSessionMaximumError) -> Self {
        match value {
            EnforceSessionMaximumError::Database(e) => e.log_500(),
        }
    }
}

#[cfg(feature = "zerver")]
impl From<InvalidJwt> for ApiError {
    fn from(value: InvalidJwt) -> Self {
        match value {
            InvalidJwt::Format => Self::UnprocessableEntity("invalid token format".to_string()),
            InvalidJwt::MissingToken => Self::UnprocessableEntity("missing token".to_string()),
            InvalidJwt::EncodingError(e) => e.log_500(),
        }
    }
}

#[cfg(feature = "zerver")]
impl From<CreateSessionError> for ApiError {
    fn from(value: CreateSessionError) -> Self {
        match value {
            CreateSessionError::Database(e) => e.log_500(),
            CreateSessionError::GetUserError(GetUserError::NotFound) => {
                Self::Unauthorized("invalid credentials".to_string())
            }
            CreateSessionError::GetUserError(e) => ApiError::from(e),
            CreateSessionError::EnforceSessionMaximumError(e) => ApiError::from(e),
            CreateSessionError::InvalidJwt(e) => e.log_500(),
        }
    }
}

#[cfg(feature = "zerver")]
impl From<RegisterUserError> for ApiError {
    fn from(value: RegisterUserError) -> Self {
        match value {
            RegisterUserError::Duplicate => Self::UnprocessableEntity(
                "user with that username or email already exists".to_string(),
            ),
            RegisterUserError::Database(e) => e.log_500(),
            RegisterUserError::FailedAccessToken(e) => e.log_500(),
            RegisterUserError::UserFromDb(e) => e.log_500(),
            RegisterUserError::CreateSessionError(e) => ApiError::from(e),
        }
    }
}

#[cfg(feature = "zerver")]
impl From<InvalidRegisterUser> for ApiError {
    fn from(value: InvalidRegisterUser) -> Self {
        match value {
            InvalidRegisterUser::Username(e) => {
                Self::UnprocessableEntity(format!("invalid username: {}", e))
            }
            InvalidRegisterUser::Email(e) => {
                Self::UnprocessableEntity(format!("invalid email: {}", e))
            }
            InvalidRegisterUser::Password(e) => {
                Self::UnprocessableEntity(format!("invalid password: {}", e))
            }
            InvalidRegisterUser::FailedPasswordHash(e) => e.log_500(),
        }
    }
}

#[cfg(feature = "zerver")]
impl TryFrom<HttpRegisterUser> for RegisterUser {
    type Error = InvalidRegisterUser;
    fn try_from(value: HttpRegisterUser) -> Result<Self, Self::Error> {
        RegisterUser::new(&value.username, &value.email, &value.password)
    }
}

/// Registers a new user and returns a session (auto-login).
#[cfg(feature = "zerver")]
pub async fn register_user(
    State(state): State<AppState>,
    Json(body): Json<HttpRegisterUser>,
) -> Result<(StatusCode, Json<Session>), ApiError> {
    let mut request = RegisterUser::new(&body.username, &body.email, &body.password)?;
    request.platform = body.platform;
    request.client_version = body.client_version;
    tracing::info!(event = "register", username = %body.username);
    let session = state
        .auth_service
        .register_user(&request)
        .await
        .map_err(ApiError::from)?;

    let user_id = session.user.id;
    let metrics = std::sync::Arc::clone(&state.metrics_service);
    tokio::spawn(async move {
        if let Err(e) = metrics.insert_lifetime_row(user_id).await {
            tracing::warn!(error = ?e, "metrics: insert_lifetime_row failed");
        }
        if let Err(e) = metrics
            .record_event(user_id, EventKind::Register, None)
            .await
        {
            tracing::warn!(error = ?e, "metrics: record register event failed");
        }
    });

    Ok((StatusCode::CREATED, session.into()))
}
