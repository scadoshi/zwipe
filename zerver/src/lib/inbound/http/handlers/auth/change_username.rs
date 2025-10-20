use crate::domain::auth::models::change_username::ChangeUsername;
#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::{
            models::change_username::{ChangeUsernameError, InvalidChangeUsername},
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
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

#[cfg(feature = "zerver")]
impl From<ChangeUsernameError> for ApiError {
    fn from(value: ChangeUsernameError) -> Self {
        match value {
            ChangeUsernameError::UserNotFound => Self::NotFound("user not found".to_string()),
            ChangeUsernameError::Database(e) => e.log_500(),
            ChangeUsernameError::UserFromDb(e) => e.log_500(),
            ChangeUsernameError::AuthenticateUserError(_) => {
                Self::Unauthorized("invalid credentials".to_string())
            }
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
            InvalidChangeUsername::Password(_) => {
                Self::Unauthorized("invalid credentials".to_string())
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HttpChangeUsername {
    new_username: String,
    password: String,
}

impl HttpChangeUsername {
    pub fn new(new_username: &str, password: &str) -> Self {
        Self {
            new_username: new_username.to_string(),
            password: password.to_string(),
        }
    }
}

impl From<ChangeUsername> for HttpChangeUsername {
    fn from(value: ChangeUsername) -> Self {
        Self {
            new_username: value.new_username.to_string(),
            password: value.password.to_string(),
        }
    }
}

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

    state
        .auth_service
        .change_username(&request)
        .await
        .map_err(ApiError::from)
        .map(|user| (StatusCode::OK, Json(user)))
}
