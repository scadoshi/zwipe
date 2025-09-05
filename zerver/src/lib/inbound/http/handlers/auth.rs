use crate::domain::deck::ports::DeckService;
use crate::inbound::http::middleware::AuthenticatedUser;
use crate::inbound::http::{ApiError, AppState};
use crate::{
    domain::{
        auth::{
            models::{
                AuthenticateUser, AuthenticateUserError, AuthenticateUserSuccess, ChangePassword,
                ChangePasswordError, InvalidAuthenticateUser, InvalidChangePassword,
                InvalidRegisterUser, RegisterUser, RegisterUserError,
            },
            ports::AuthService,
        },
        card::ports::CardService,
        health::ports::HealthService,
        user::ports::UserService,
    },
    inbound::http::Log500,
};
use axum::{extract::State, http::StatusCode, Json};
use serde::Deserialize;

// ==========
//  register
// ==========

impl From<RegisterUserError> for ApiError {
    fn from(value: RegisterUserError) -> Self {
        match value {
            RegisterUserError::Duplicate => Self::UnprocessableEntity(
                "user with that username or email already exists".to_string(),
            ),

            e => e.log_500(),
        }
    }
}

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
                Self::UnprocessableEntity(format!("invalid password {}", e))
            }
            e => e.log_500(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RegisterUserBody {
    username: String,
    email: String,
    password: String,
}

impl TryFrom<RegisterUserBody> for RegisterUser {
    type Error = InvalidRegisterUser;
    fn try_from(value: RegisterUserBody) -> Result<Self, Self::Error> {
        RegisterUser::new(&value.username, &value.email, &value.password)
    }
}

pub async fn register_user<AS, US, HS, CS, DS>(
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Json(body): Json<RegisterUserBody>,
) -> Result<(StatusCode, Json<AuthenticateUserSuccess>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let req = RegisterUser::new(&body.username, &body.email, &body.password)?;

    state
        .auth_service
        .register_user(&req)
        .await
        .map_err(ApiError::from)
        .map(|response| (StatusCode::CREATED, response.into()))
}

// ==============
//  authenticate
// ==============

impl From<AuthenticateUserError> for ApiError {
    fn from(value: AuthenticateUserError) -> Self {
        match value {
            AuthenticateUserError::UserNotFound => {
                Self::Unauthorized("invalid credentials".to_string())
            }

            AuthenticateUserError::InvalidPassword => {
                Self::Unauthorized("invalid credentials".to_string())
            }

            e => e.log_500(),
        }
    }
}

impl From<InvalidAuthenticateUser> for ApiError {
    fn from(value: InvalidAuthenticateUser) -> Self {
        match value {
            InvalidAuthenticateUser::MissingIdentifier => {
                Self::UnprocessableEntity("username or email must be present".to_string())
            }
            InvalidAuthenticateUser::MissingPassword => {
                Self::UnprocessableEntity("password must be present".to_string())
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct HttpAuthenticateUser {
    identifier: String,
    password: String,
}

impl TryFrom<HttpAuthenticateUser> for AuthenticateUser {
    type Error = InvalidAuthenticateUser;
    fn try_from(value: HttpAuthenticateUser) -> Result<Self, Self::Error> {
        AuthenticateUser::new(&value.identifier, &value.password)
    }
}

pub async fn authenticate_user<AS, US, HS, CS, DS>(
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Json(body): Json<HttpAuthenticateUser>,
) -> Result<(StatusCode, Json<AuthenticateUserSuccess>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let req = AuthenticateUser::new(&body.identifier, &body.password)?;

    state
        .auth_service
        .authenticate_user(&req)
        .await
        .map_err(ApiError::from)
        .map(|response| (StatusCode::OK, response.into()))
}

// =================
//  change password
// =================

impl From<ChangePasswordError> for ApiError {
    fn from(value: ChangePasswordError) -> Self {
        match value {
            ChangePasswordError::UserNotFound => {
                Self::UnprocessableEntity("user not found".to_string())
            }
            e => e.log_500(),
        }
    }
}

impl From<InvalidChangePassword> for ApiError {
    fn from(value: InvalidChangePassword) -> Self {
        match value {
            InvalidChangePassword::InvalidId(e) => {
                Self::UnprocessableEntity(format!("invalid id {}", e))
            }
            InvalidChangePassword::PasswordError(e) => {
                Self::UnprocessableEntity(format!("invalid password {}", e))
            }
            e => e.log_500(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct HttpChangePassword {
    id: String,
    new_password: String,
}

impl TryFrom<HttpChangePassword> for ChangePassword {
    type Error = InvalidChangePassword;
    fn try_from(value: HttpChangePassword) -> Result<Self, Self::Error> {
        ChangePassword::new(&value.id, &value.new_password)
    }
}

pub async fn change_password<AS, US, HS, CS, DS>(
    _: AuthenticatedUser,
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
    let req = ChangePassword::new(&body.id, &body.new_password)?;

    state
        .auth_service
        .change_password(&req)
        .await
        .map_err(ApiError::from)
        .map(|_| (StatusCode::OK, Json(())))
}
