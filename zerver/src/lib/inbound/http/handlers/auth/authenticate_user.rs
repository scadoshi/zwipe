#[cfg(feature = "zerver")]
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::domain::auth::models::authenticate_user::AuthenticateUser;
#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::{
            models::{
                authenticate_user::{AuthenticateUserError, InvalidAuthenticateUser},
                session::Session,
            },
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
impl From<AuthenticateUserError> for ApiError {
    fn from(value: AuthenticateUserError) -> Self {
        match value {
            AuthenticateUserError::UserNotFound | AuthenticateUserError::InvalidPassword => {
                Self::Unauthorized("invalid credentials".to_string())
            }
            AuthenticateUserError::Database(e) => e.log_500(),
            AuthenticateUserError::UserFromDb(e) => e.log_500(),
            AuthenticateUserError::FailedToVerify(e) => e.log_500(),
            AuthenticateUserError::FailedAccessToken(e) => e.log_500(),
            AuthenticateUserError::CreateSessionError(e) => ApiError::from(e),
        }
    }
}

#[cfg(feature = "zerver")]
impl From<InvalidAuthenticateUser> for ApiError {
    fn from(value: InvalidAuthenticateUser) -> Self {
        match value {
            InvalidAuthenticateUser::MissingIdentifier | InvalidAuthenticateUser::Password(_) => {
                Self::UnprocessableEntity("invalid credentials".to_string())
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HttpAuthenticateUser {
    identifier: String,
    password: String,
}

impl HttpAuthenticateUser {
    pub fn new(identifier: &str, password: &str) -> Self {
        Self {
            identifier: identifier.to_string(),
            password: password.to_string(),
        }
    }
}

impl From<AuthenticateUser> for HttpAuthenticateUser {
    fn from(value: AuthenticateUser) -> Self {
        Self {
            identifier: value.identifier,
            password: value.password,
        }
    }
}

#[cfg(feature = "zerver")]
impl TryFrom<HttpAuthenticateUser> for AuthenticateUser {
    type Error = InvalidAuthenticateUser;
    fn try_from(value: HttpAuthenticateUser) -> Result<Self, Self::Error> {
        AuthenticateUser::new(&value.identifier, &value.password)
    }
}

#[cfg(feature = "zerver")]
pub async fn authenticate_user<AS, US, HS, CS, DS>(
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Json(body): Json<HttpAuthenticateUser>,
) -> Result<(StatusCode, Json<Session>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let request = AuthenticateUser::new(&body.identifier, &body.password)?;

    state
        .auth_service
        .authenticate_user(&request)
        .await
        .map_err(ApiError::from)
        .map(|response| (StatusCode::OK, response.into()))
}
