#[cfg(feature = "zerver")]
use crate::inbound::http::Log500;
#[cfg(feature = "zerver")]
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::domain::auth::models::register_user::RawRegisterUser;
#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::{
            models::{
                access_token::InvalidJwt,
                register_user::{InvalidRegisterUser, RegisterUser, RegisterUserError},
                session::{
                    create_session::CreateSessionError,
                    enforce_session_maximum::EnforceSessionMaximumError, Session,
                },
            },
            ports::AuthService,
        },
        card::ports::CardService,
        deck::ports::DeckService,
        health::ports::HealthService,
        user::{models::get_user::GetUserError, ports::UserService},
    },
    inbound::http::{ApiError, AppState},
};

#[cfg(feature = "zerver")]
impl From<EnforceSessionMaximumError> for ApiError {
    fn from(value: EnforceSessionMaximumError) -> Self {
        match value {
            EnforceSessionMaximumError::Database(e) => e.log_500(),
        }
    }
}

#[cfg(feature = "zerver")]
impl From<InvalidJwt> for ApiError {
    fn from(value: InvalidJwt) -> Self {
        match value {
            InvalidJwt::Format => Self::UnprocessableEntity("invalid token format".to_string()),
            InvalidJwt::MissingToken => Self::UnprocessableEntity("missing token".to_string()),
            InvalidJwt::EncodingError(e) => e.log_500(),
        }
    }
}

#[cfg(feature = "zerver")]
impl From<CreateSessionError> for ApiError {
    fn from(value: CreateSessionError) -> Self {
        match value {
            CreateSessionError::Database(e) => e.log_500(),
            CreateSessionError::GetUserError(GetUserError::NotFound) => {
                Self::Unauthorized("invalid credentials".to_string())
            }
            CreateSessionError::GetUserError(e) => ApiError::from(e),
            CreateSessionError::EnforceSessionMaximumError(e) => ApiError::from(e),
            CreateSessionError::InvalidJwt(e) => e.log_500(),
        }
    }
}

#[cfg(feature = "zerver")]
impl From<RegisterUserError> for ApiError {
    fn from(value: RegisterUserError) -> Self {
        match value {
            RegisterUserError::Duplicate => Self::UnprocessableEntity(
                "user with that username or email already exists".to_string(),
            ),
            RegisterUserError::Database(e) => e.log_500(),
            RegisterUserError::FailedAccessToken(e) => e.log_500(),
            RegisterUserError::UserFromDb(e) => e.log_500(),
            RegisterUserError::CreateSessionError(e) => ApiError::from(e),
        }
    }
}

#[cfg(feature = "zerver")]
impl From<InvalidRegisterUser> for ApiError {
    fn from(value: InvalidRegisterUser) -> Self {
        match value {
            InvalidRegisterUser::Username(e) => {
                Self::UnprocessableEntity(format!("invalid username: {}", e))
            }
            InvalidRegisterUser::Email(e) => {
                Self::UnprocessableEntity(format!("invalid email: {}", e))
            }
            InvalidRegisterUser::Password(e) => {
                Self::UnprocessableEntity(format!("invalid password: {}", e))
            }
            InvalidRegisterUser::FailedPasswordHash(e) => e.log_500(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HttpRegisterUser {
    username: String,
    email: String,
    password: String,
}

impl HttpRegisterUser {
    pub fn new(username: &str, email: &str, password: &str) -> Self {
        Self {
            username: username.to_string(),
            email: email.to_string(),
            password: password.to_string(),
        }
    }
}

#[cfg(feature = "zerver")]
impl TryFrom<HttpRegisterUser> for RegisterUser {
    type Error = InvalidRegisterUser;
    fn try_from(value: HttpRegisterUser) -> Result<Self, Self::Error> {
        RegisterUser::new(&value.username, &value.email, &value.password)
    }
}

impl From<RawRegisterUser> for HttpRegisterUser {
    fn from(value: RawRegisterUser) -> Self {
        Self::new(
            &value.username.to_string(),
            &value.email.to_string(),
            &value.password.read().to_string(),
        )
    }
}

#[cfg(feature = "zerver")]
pub async fn register_user<AS, US, HS, CS, DS>(
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Json(body): Json<HttpRegisterUser>,
) -> Result<(StatusCode, Json<Session>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let request = RegisterUser::new(&body.username, &body.email, &body.password)?;

    state
        .auth_service
        .register_user(&request)
        .await
        .map_err(ApiError::from)
        .map(|response| (StatusCode::CREATED, response.into()))
}
