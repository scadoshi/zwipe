use crate::domain::auth::models::access_token::InvalidJwt;
use crate::domain::auth::models::session::RefreshSession;
#[cfg(feature = "zerver")]
use crate::domain::auth::models::session::{
    CreateSessionError, EnforceSessionMaximumError, InvalidRefreshSession, RefreshSessionError,
};
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
                session::Session, AuthenticateUserError, ChangePassword, ChangePasswordError,
                InvalidAuthenticateUser, InvalidChangePassword, InvalidRegisterUser, RegisterUser,
                RegisterUserError,
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
            CreateSessionError::GetUserError(e) => ApiError::from(e),
            CreateSessionError::EnforceSessionMaximumError(e) => ApiError::from(e),
            CreateSessionError::InvalidJwt(e) => ApiError::from(e),
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

// ==============
//  authenticate
// ==============

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

// =========
//  refresh
// =========

#[cfg(feature = "zerver")]
impl From<RefreshSessionError> for ApiError {
    fn from(value: RefreshSessionError) -> Self {
        match value {
            RefreshSessionError::CreateSessionError(e) => ApiError::from(e),
            RefreshSessionError::Database(e) => e.log_500(),
            RefreshSessionError::GetUserError(e) => e.log_500(),
            RefreshSessionError::InvalidJwt(e) => ApiError::from(e),
            RefreshSessionError::EnforceSessionMaximumError(e) => ApiError::from(e),
            RefreshSessionError::NotFound(u) => {
                tracing::info!("{}", RefreshSessionError::NotFound(u).to_string());
                Self::Unauthorized("invalid refresh token".to_string())
            }
            RefreshSessionError::Expired(u) => {
                tracing::info!("{}", RefreshSessionError::Expired(u).to_string());
                Self::Unauthorized("refresh token expired".to_string())
            }
            RefreshSessionError::Revoked(u) => {
                tracing::warn!("{}", RefreshSessionError::Revoked(u).to_string());
                Self::Unauthorized("invalid refresh token".to_string())
            }
            RefreshSessionError::Forbidden(u) => {
                tracing::warn!("{}", RefreshSessionError::Forbidden(u).to_string());
                Self::Forbidden("invalid refresh token".to_string())
            }
        }
    }
}

#[cfg(feature = "zerver")]
impl From<InvalidRefreshSession> for ApiError {
    fn from(value: InvalidRefreshSession) -> Self {
        match value {
            InvalidRefreshSession::UserId(_) => {
                Self::UnprocessableEntity("invalid user id".to_string())
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HttpRefreshSession {
    user_id: String,
    refresh_token: String,
}

impl HttpRefreshSession {
    pub fn new(user_id: &str, refresh_token: &str) -> Self {
        Self {
            user_id: user_id.to_string(),
            refresh_token: refresh_token.to_string(),
        }
    }
}

impl From<RefreshSession> for HttpRefreshSession {
    fn from(value: RefreshSession) -> Self {
        Self {
            user_id: value.user_id.to_string(),
            refresh_token: value.refresh_token,
        }
    }
}

#[cfg(feature = "zerver")]
impl TryFrom<HttpRefreshSession> for RefreshSession {
    type Error = InvalidRefreshSession;
    fn try_from(value: HttpRefreshSession) -> Result<Self, Self::Error> {
        Self::new(&value.user_id, &value.refresh_token)
    }
}

#[cfg(feature = "zerver")]
pub async fn refresh_session<AS, US, HS, CS, DS>(
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Json(body): Json<HttpRefreshSession>,
) -> Result<(StatusCode, Json<Session>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let request = RefreshSession::new(&body.user_id, &body.refresh_token)?;

    state
        .auth_service
        .refresh_session(&request)
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

// =================
//  change username
// =================

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

// ==============
//  change email
// ==============

#[cfg(feature = "zerver")]
impl From<ChangeEmailError> for ApiError {
    fn from(value: ChangeEmailError) -> Self {
        match value {
            ChangeEmailError::UserNotFound => Self::NotFound("user not found".to_string()),
            ChangeEmailError::Database(e) => e.log_500(),
            ChangeEmailError::UserFromDb(e) => e.log_500(),
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
            DeleteUserError::Database(e) => e.log_500(),
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
