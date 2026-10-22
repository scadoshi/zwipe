use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;
use zwipe_core::http::contracts::auth::{
    HttpRegisterUser, HttpRegistrationAccepted, RegistrationStatus,
};
use zwipe_core::version::canonical_version;

use crate::domain::auth::models::access_token::InvalidJwt;
use crate::domain::auth::requests::create_session::CreateSessionError;
use crate::domain::auth::requests::enforce_session_maximum::EnforceSessionMaximumError;
use crate::domain::auth::requests::register_user::{
    InvalidRegisterUser, RegisterOutcome, RegisterUser, RegisterUserError,
};
use crate::domain::metrics::models::kinds::EventKind;
use crate::domain::user::models::get_user::GetUserError;
use crate::inbound::http::{ApiError, AppState, To500};

impl From<EnforceSessionMaximumError> for ApiError {
    fn from(value: EnforceSessionMaximumError) -> Self {
        match value {
            EnforceSessionMaximumError::Database(e) => e.to_500(),
        }
    }
}

impl From<InvalidJwt> for ApiError {
    fn from(value: InvalidJwt) -> Self {
        match value {
            InvalidJwt::Format => Self::UnprocessableEntity("invalid token format".to_string()),
            InvalidJwt::MissingToken => Self::UnprocessableEntity("missing token".to_string()),
            InvalidJwt::EncodingError(e) => e.to_500(),
        }
    }
}

impl From<CreateSessionError> for ApiError {
    fn from(value: CreateSessionError) -> Self {
        match value {
            CreateSessionError::Database(e) => e.to_500(),
            CreateSessionError::GetUserError(GetUserError::NotFound) => {
                Self::Unauthorized("invalid credentials".to_string())
            }
            CreateSessionError::GetUserError(e) => ApiError::from(e),
            CreateSessionError::EnforceSessionMaximumError(e) => ApiError::from(e),
            CreateSessionError::InvalidJwt(e) => e.to_500(),
        }
    }
}

impl From<RegisterUserError> for ApiError {
    fn from(value: RegisterUserError) -> Self {
        match value {
            // The old path's answer, unchanged until clients older than 409
            // are below the version floor.
            RegisterUserError::Duplicate | RegisterUserError::DuplicateEmail => {
                Self::UnprocessableEntity(
                    "user with that username or email already exists".to_string(),
                )
            }
            // Only the verify_first path checks this, and only clients that
            // know 409 send verify_first.
            RegisterUserError::UsernameTaken => {
                Self::Conflict("username already taken".to_string())
            }
            RegisterUserError::Database(e) => e.to_500(),
            RegisterUserError::FailedAccessToken(e) => e.to_500(),
            RegisterUserError::UserFromDb(e) => e.to_500(),
            RegisterUserError::CreateSessionError(e) => ApiError::from(e),
        }
    }
}

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
            InvalidRegisterUser::FailedPasswordHash(e) => e.to_500(),
        }
    }
}

impl TryFrom<HttpRegisterUser> for RegisterUser {
    type Error = InvalidRegisterUser;
    fn try_from(value: HttpRegisterUser) -> Result<Self, Self::Error> {
        RegisterUser::new(&value.username, &value.email, value.password.read())
    }
}

/// Registers a new user.
///
/// With `verify_first`, answers `202` and the same body whether the email is
/// new or already has an account; the caller logs in afterwards. Without it,
/// answers `201` with a session, which is what clients from before the field
/// get.
pub async fn register_user(
    State(state): State<AppState>,
    Json(body): Json<HttpRegisterUser>,
) -> Result<Response, ApiError> {
    let mut request = RegisterUser::new(&body.username, &body.email, body.password.read())?;
    request.platform = body.platform;
    request.client_version = body.client_version.as_deref().and_then(canonical_version);
    tracing::info!(event = "register", username = %body.username, verify_first = body.verify_first);

    if body.verify_first {
        let outcome = state
            .auth_service
            .register_user_verify_first(&request)
            .await
            .map_err(ApiError::from)?;
        if let RegisterOutcome::Created(user) = outcome {
            record_registration(&state, user.id);
        }
        let accepted = HttpRegistrationAccepted {
            status: RegistrationStatus::VerificationSent,
        };
        return Ok((StatusCode::ACCEPTED, Json(accepted)).into_response());
    }

    let session = state
        .auth_service
        .register_user(&request)
        .await
        .map_err(ApiError::from)?;
    record_registration(&state, session.user.id);

    Ok((StatusCode::CREATED, Json(session)).into_response())
}

/// Starts the new account's metrics rows in the background.
fn record_registration(state: &AppState, user_id: Uuid) {
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
}
