#[cfg(feature = "zerver")]
use axum::{extract::State, http::StatusCode, Json};
pub use zwipe_core::http::contracts::auth::HttpResetPassword;

#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::{
            requests::reset_password::{ResetPassword, ResetPasswordError},
            ports::AuthService,
        },
        card::ports::CardService,
        deck::ports::DeckService,
        health::ports::HealthService,
        user::ports::UserService,
    },
    inbound::http::{ApiError, AppState, Log500},
};

#[cfg(feature = "zerver")]
impl From<ResetPasswordError> for ApiError {
    fn from(value: ResetPasswordError) -> Self {
        match value {
            ResetPasswordError::InvalidToken => {
                Self::Unauthorized("token not found or expired".to_string())
            }
            ResetPasswordError::InvalidPassword(msg) => Self::UnprocessableEntity(msg),
            ResetPasswordError::Database(e) => e.log_500(),
        }
    }
}

/// Completes the password reset flow using a one-time token.
///
/// Revokes all existing sessions after a successful reset.
#[cfg(feature = "zerver")]
pub async fn reset_password<AS, US, HS, CS, DS>(
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Json(body): Json<HttpResetPassword>,
) -> Result<StatusCode, ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let request = ResetPassword::new(body.token, &body.new_password)?;
    state
        .auth_service
        .reset_password(&request)
        .await
        .map_err(ApiError::from)
        .map(|_| StatusCode::OK)
}
