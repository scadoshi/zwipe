#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::{
            ports::AuthService,
            requests::change_username::{ChangeUsername, ChangeUsernameError, InvalidChangeUsername},
        },
        card::ports::CardService,
        deck::ports::DeckService,
        health::ports::HealthService,
        metrics::models::kinds::AuditAction,
        user::ports::UserService,
    },
    inbound::http::{middleware::AuthenticatedUser, ApiError, AppState, Log500},
};
#[cfg(feature = "zerver")]
use axum::{extract::State, http::StatusCode, Json};
#[cfg(feature = "zerver")]
use zwipe_core::http::contracts::auth::HttpChangeUsername;
#[cfg(feature = "zerver")]
use zwipe_core::domain::user::User;

#[cfg(feature = "zerver")]
impl From<ChangeUsernameError> for ApiError {
    fn from(value: ChangeUsernameError) -> Self {
        tracing::warn!(event = "change_username_error", error = %value);
        match value {
            ChangeUsernameError::UserNotFound => Self::NotFound("user not found".to_string()),
            ChangeUsernameError::Duplicate => {
                Self::UnprocessableEntity("username already in use".to_string())
            }
            ChangeUsernameError::Database(e) => e.log_500(),
            ChangeUsernameError::UserFromDb(e) => e.log_500(),
            ChangeUsernameError::AuthenticateUserError(e) => ApiError::from(e),
        }
    }
}

#[cfg(feature = "zerver")]
impl From<InvalidChangeUsername> for ApiError {
    fn from(value: InvalidChangeUsername) -> Self {
        match value {
            InvalidChangeUsername::Id(e) => Self::UnprocessableEntity(format!("invalid id: {}", e)),
            InvalidChangeUsername::Username(e) => {
                Self::UnprocessableEntity(format!("invalid username: {}", e))
            }
        }
    }
}

/// Changes the user's username after verifying the password.
#[cfg(feature = "zerver")]
pub async fn change_username<AS, US, HS, CS, DS>(
    user: AuthenticatedUser,
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Json(body): Json<HttpChangeUsername>,
) -> Result<(StatusCode, Json<User>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let request = ChangeUsername::new(user.id, &body.new_username, &body.password)?;

    let updated = state
        .auth_service
        .change_username(&request)
        .await
        .map_err(ApiError::from)?;

    let metrics = std::sync::Arc::clone(&state.metrics_service);
    let uid = user.id;
    tokio::spawn(async move {
        if let Err(e) = metrics.record_audit(uid, AuditAction::UsernameChanged).await {
            tracing::warn!(error = ?e, "metrics: audit username change failed");
        }
    });

    Ok((StatusCode::OK, Json(updated)))
}
