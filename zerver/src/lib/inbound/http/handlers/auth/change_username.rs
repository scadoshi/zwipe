#[cfg(feature = "zerver")]
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::{
            models::change_username::{ChangeUsername, ChangeUsernameError, InvalidChangeUsername},
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
impl From<ChangeUsernameError> for ApiError {
    fn from(value: ChangeUsernameError) -> Self {
        match value {
            ChangeUsernameError::UserNotFound => Self::NotFound("user not found".to_string()),
            ChangeUsernameError::Database(e) => e.log_500(),
            ChangeUsernameError::UserFromDb(e) => e.log_500(),
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

#[derive(Debug, Deserialize, Serialize)]
pub struct HttpChangeUsername {
    username: String,
}

impl HttpChangeUsername {
    pub fn new(username: &str) -> Self {
        Self {
            username: username.to_string(),
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
    let request = ChangeUsername::new(user.id, &body.username)?;

    state
        .auth_service
        .change_username(&request)
        .await
        .map_err(ApiError::from)
        .map(|user| (StatusCode::OK, Json(user)))
}
