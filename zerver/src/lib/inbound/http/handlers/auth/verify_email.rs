#[cfg(feature = "zerver")]
use axum::{extract::State, http::StatusCode, Json};
#[cfg(feature = "zerver")]
use zwipe_core::http::contracts::auth::HttpVerifyEmail;

#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::{
            requests::verify_email::{VerifyEmail, VerifyEmailError},
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
impl From<VerifyEmailError> for ApiError {
    fn from(value: VerifyEmailError) -> Self {
        match value {
            VerifyEmailError::InvalidToken => {
                Self::UnprocessableEntity("token not found or expired".to_string())
            }
            VerifyEmailError::Database(e) => e.log_500(),
        }
    }
}

/// Verifies a user's email address using a one-time token.
#[cfg(feature = "zerver")]
pub async fn verify_email<AS, US, HS, CS, DS>(
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Json(body): Json<HttpVerifyEmail>,
) -> Result<StatusCode, ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let request = VerifyEmail {
        token: body.token,
    };
    state
        .auth_service
        .verify_email(&request)
        .await
        .map_err(ApiError::from)
        .map(|_| StatusCode::OK)
}
