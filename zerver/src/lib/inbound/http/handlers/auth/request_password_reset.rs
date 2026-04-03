#[cfg(feature = "zerver")]
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::{
            requests::request_password_reset::{RequestPasswordReset, RequestPasswordResetError},
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
impl From<RequestPasswordResetError> for ApiError {
    fn from(value: RequestPasswordResetError) -> Self {
        match value {
            RequestPasswordResetError::Database(e) => e.log_500(),
        }
    }
}

/// Password reset initiation request body.
#[derive(Debug, Deserialize, Serialize)]
pub struct HttpRequestPasswordReset {
    email: String,
}

impl HttpRequestPasswordReset {
    /// Creates a new password reset request for the given email address.
    pub fn new(email: &str) -> Self {
        Self { email: email.to_string() }
    }
}

/// Initiates the password reset flow for the given email address.
///
/// Always returns `200 OK` regardless of whether the email is registered,
/// to prevent email enumeration attacks.
#[cfg(feature = "zerver")]
pub async fn request_password_reset<AS, US, HS, CS, DS>(
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Json(body): Json<HttpRequestPasswordReset>,
) -> Result<StatusCode, ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let request = RequestPasswordReset { email: body.email };
    state
        .auth_service
        .request_password_reset(&request)
        .await
        .map_err(ApiError::from)
        .map(|_| StatusCode::OK)
}
