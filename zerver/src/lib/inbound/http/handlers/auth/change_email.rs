#[cfg(feature = "zerver")]
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::{
            models::change_email::{ChangeEmail, ChangeEmailError, InvalidChangeEmail},
            ports::AuthService,
        },
        card::ports::CardService,
        deck::ports::DeckService,
        health::ports::HealthService,
        user::{models::User, ports::UserService},
    },
    inbound::http::{middleware::AuthenticatedUser, ApiError, AppState, Log500},
};

#[cfg(feature = "zerver")]
impl From<ChangeEmailError> for ApiError {
    fn from(value: ChangeEmailError) -> Self {
        match value {
            ChangeEmailError::UserNotFound => Self::NotFound("user not found".to_string()),
            ChangeEmailError::Database(e) => e.log_500(),
            ChangeEmailError::UserFromDb(e) => e.log_500(),
            ChangeEmailError::AuthenticateUserError(_) => {
                Self::Unauthorized("invalid credentials".to_string())
            }
            ChangeEmailError::Duplicate => {
                Self::UnprocessableEntity("email already in use".to_string())
            }
        }
    }
}

#[cfg(feature = "zerver")]
impl From<InvalidChangeEmail> for ApiError {
    fn from(value: InvalidChangeEmail) -> Self {
        match value {
            InvalidChangeEmail::Id(e) => Self::UnprocessableEntity(format!("invalid id: {}", e)),
            InvalidChangeEmail::Email(e) => {
                Self::UnprocessableEntity(format!("invalid email: {}", e))
            }
            InvalidChangeEmail::Password(_) => {
                Self::Unauthorized("invalid credentials".to_string())
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HttpChangeEmail {
    email: String,
    password: String,
}

impl HttpChangeEmail {
    pub fn new(email: &str, password: &str) -> Self {
        Self {
            email: email.to_string(),
            password: password.to_string(),
        }
    }
}

#[cfg(feature = "zerver")]
pub async fn change_email<AS, US, HS, CS, DS>(
    user: AuthenticatedUser,
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Json(body): Json<HttpChangeEmail>,
) -> Result<(StatusCode, Json<User>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let request = ChangeEmail::new(user.id, &body.email, &body.password)?;

    state
        .auth_service
        .change_email(&request)
        .await
        .map_err(ApiError::from)
        .map(|user| (StatusCode::OK, Json(user)))
}
