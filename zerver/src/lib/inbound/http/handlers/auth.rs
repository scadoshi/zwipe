use crate::domain::auth::models::{AuthenticateUser, RawRegisterUser};
#[cfg(feature = "zerver")]
use crate::domain::auth::models::{
    ChangeEmail, ChangeEmailError, ChangeUsername, ChangeUsernameError, DeleteUser,
    DeleteUserError, InvalidChangeEmail, InvalidChangeUsername,
};
#[cfg(feature = "zerver")]
use crate::domain::deck::ports::DeckService;
#[cfg(feature = "zerver")]
use crate::domain::user::models::User;
#[cfg(feature = "zerver")]
use crate::inbound::http::middleware::AuthenticatedUser;
#[cfg(feature = "zerver")]
use crate::inbound::http::{ApiError, AppState};
#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::{
            models::{
                AuthenticateUserError, AuthenticateUserSuccess, ChangePassword,
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
#[cfg(feature = "zerver")]
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

// ==========
//  register
// ==========

#[cfg(feature = "zerver")]
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
            e => e.log_500(),
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

#[cfg(feature = "zerver")]
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

#[cfg(feature = "zerver")]
impl From<InvalidAuthenticateUser> for ApiError {
    fn from(value: InvalidAuthenticateUser) -> Self {
        match value {
            InvalidAuthenticateUser::MissingIdentifier => {
                Self::UnprocessableEntity("username or email must be present".to_string())
            }
            InvalidAuthenticateUser::Password(_) => {
                Self::UnprocessableEntity("invalid username, email or password".to_string())
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

#[cfg(feature = "zerver")]
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

#[cfg(feature = "zerver")]
impl From<InvalidChangePassword> for ApiError {
    fn from(value: InvalidChangePassword) -> Self {
        match value {
            InvalidChangePassword::Password(e) => {
                Self::UnprocessableEntity(format!("invalid password {}", e))
            }
            e => e.log_500(),
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

// =================
//  change username
// =================

#[cfg(feature = "zerver")]
impl From<ChangeUsernameError> for ApiError {
    fn from(value: ChangeUsernameError) -> Self {
        match value {
            ChangeUsernameError::UserNotFound => Self::NotFound("user not found".to_string()),
            e => e.log_500(),
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

// ==============
//  change email
// ==============

#[cfg(feature = "zerver")]
impl From<ChangeEmailError> for ApiError {
    fn from(value: ChangeEmailError) -> Self {
        match value {
            ChangeEmailError::UserNotFound => Self::NotFound("user not found".to_string()),
            e => e.log_500(),
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
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HttpChangeEmail {
    email: String,
}

impl HttpChangeEmail {
    pub fn new(email: &str) -> Self {
        Self {
            email: email.to_string(),
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
    let request = ChangeEmail::new(user.id, &body.email)?;

    state
        .auth_service
        .change_email(&request)
        .await
        .map_err(ApiError::from)
        .map(|user| (StatusCode::OK, Json(user)))
}

// ========
//  delete
// ========

#[cfg(feature = "zerver")]
impl From<DeleteUserError> for ApiError {
    fn from(value: DeleteUserError) -> Self {
        match value {
            DeleteUserError::NotFound => Self::NotFound("user not found".to_string()),
            e => e.log_500(),
        }
    }
}

#[cfg(feature = "zerver")]
pub async fn delete_user<AS, US, HS, CS, DS>(
    user: AuthenticatedUser,
    State(state): State<AppState<AS, US, HS, CS, DS>>,
) -> Result<StatusCode, ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let request = DeleteUser::from(user.id);

    state
        .auth_service
        .delete_user(&request)
        .await
        .map_err(ApiError::from)
        .map(|_| StatusCode::NO_CONTENT)
}
