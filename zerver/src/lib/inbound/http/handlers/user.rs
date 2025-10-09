#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::ports::AuthService,
        card::ports::CardService,
        deck::ports::DeckService,
        health::ports::HealthService,
        user::{
            models::{GetUser, GetUserError, User},
            ports::UserService,
        },
    },
    inbound::http::{middleware::AuthenticatedUser, ApiError, AppState, Log500},
};
#[cfg(feature = "zerver")]
use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;

// =====
//  get
// =====

#[cfg(feature = "zerver")]
impl From<GetUserError> for ApiError {
    fn from(value: GetUserError) -> Self {
        match value {
            GetUserError::NotFound => Self::NotFound("user not found".to_string()),
            GetUserError::Database(e) => e.log_500(),
            GetUserError::UserFromDb(e) => e.log_500(),
        }
    }
}

#[cfg(feature = "zerver")]
impl From<AuthenticatedUser> for GetUser {
    fn from(value: AuthenticatedUser) -> Self {
        GetUser::from(value.id)
    }
}

#[cfg(feature = "zerver")]
pub async fn get_user<AS, US, HS, CS, DS>(
    user: AuthenticatedUser,
    State(state): State<AppState<AS, US, HS, CS, DS>>,
) -> Result<(StatusCode, Json<HttpUser>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let request = GetUser::from(user);

    state
        .user_service
        .get_user(&request)
        .await
        .map_err(ApiError::from)
        .map(|user| (StatusCode::OK, Json(user.into())))
}

// ==========
//  response
// ==========

/// for returning `User` data from methods
///
/// create, get and update use this
#[derive(Debug, Serialize, PartialEq)]
pub struct HttpUser {
    id: String,
    username: String,
    email: String,
}

impl HttpUser {
    pub fn new(id: &str, username: &str, email: &str) -> Self {
        Self {
            id: id.to_string(),
            username: username.to_string(),
            email: email.to_string(),
        }
    }
}

#[cfg(feature = "zerver")]
impl From<User> for HttpUser {
    fn from(user: User) -> Self {
        Self {
            id: user.id.to_string(),
            username: user.username.to_string(),
            email: user.email.to_string(),
        }
    }
}
