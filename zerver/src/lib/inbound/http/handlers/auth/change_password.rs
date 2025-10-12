#[cfg(feature = "zerver")]
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::{
            models::change_password::{ChangePassword, ChangePasswordError, InvalidChangePassword},
            ports::AuthService,
        },
        card::ports::CardService,
        deck::ports::DeckService,
        health::ports::HealthService,
        user::ports::UserService,
    },
    inbound::http::{middleware::AuthenticatedUser, ApiError, AppState, Log500},
};

#[cfg(feature = "zerver")]
impl From<ChangePasswordError> for ApiError {
    fn from(value: ChangePasswordError) -> Self {
        match value {
            ChangePasswordError::UserNotFound => {
                Self::UnprocessableEntity("user not found".to_string())
            }
            ChangePasswordError::Database(e) => e.log_500(),
            ChangePasswordError::AuthenticateUserError(e) => ApiError::from(e),
        }
    }
}

#[cfg(feature = "zerver")]
impl From<InvalidChangePassword> for ApiError {
    fn from(value: InvalidChangePassword) -> Self {
        match value {
            InvalidChangePassword::Password(e) => {
                Self::UnprocessableEntity(format!("invalid password {}", e))
            }
            InvalidChangePassword::FailedPasswordHash(e) => e.log_500(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HttpChangePassword {
    current_password: String,
    new_password: String,
}

impl HttpChangePassword {
    pub fn new(current_password: &str, new_password: &str) -> Self {
        Self {
            current_password: current_password.to_string(),
            new_password: new_password.to_string(),
        }
    }
}

#[cfg(feature = "zerver")]
pub async fn change_password<AS, US, HS, CS, DS>(
    user: AuthenticatedUser,
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Json(body): Json<HttpChangePassword>,
) -> Result<(StatusCode, Json<()>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let request = ChangePassword::new(user.id, &body.current_password, &body.new_password)?;

    state
        .auth_service
        .change_password(&request)
        .await
        .map_err(ApiError::from)
        .map(|_| (StatusCode::OK, Json(())))
}
