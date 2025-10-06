pub mod bad_words;
pub mod jwt;
pub mod password;
use crate::domain::auth::models::jwt::Jwt;
#[cfg(feature = "zerver")]
use crate::domain::auth::models::jwt::JwtError;
#[cfg(feature = "zerver")]
use crate::domain::auth::models::password::HashedPassword;
use crate::domain::auth::models::password::{InvalidPassword, Password};
use crate::domain::user::models::{InvalidUsername, User, Username};
use email_address::EmailAddress;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;
use uuid::Uuid;

// ========
//  errors
// ========

#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum RegisterUserError {
    #[error("user with name or email already exists")]
    Duplicate,
    #[error(transparent)]
    Database(anyhow::Error),
    #[error("failed to generate `Jwt`: {0}")]
    FailedJwt(anyhow::Error),
    #[error("user created but database returned invalid object: {0}")]
    UserFromDb(anyhow::Error),
}

#[derive(Debug, Error)]
pub enum InvalidRawRegisterUser {
    #[error(transparent)]
    Username(InvalidUsername),
    #[error(transparent)]
    Email(email_address::Error),
    #[error(transparent)]
    Password(InvalidPassword),
}

impl From<InvalidUsername> for InvalidRawRegisterUser {
    fn from(value: InvalidUsername) -> Self {
        Self::Username(value)
    }
}

impl From<email_address::Error> for InvalidRawRegisterUser {
    fn from(value: email_address::Error) -> Self {
        Self::Email(value)
    }
}

impl From<InvalidPassword> for InvalidRawRegisterUser {
    fn from(value: InvalidPassword) -> Self {
        Self::Password(value)
    }
}

#[derive(Debug, Error)]
pub enum InvalidRegisterUser {
    #[error(transparent)]
    Username(InvalidUsername),
    #[error(transparent)]
    Email(email_address::Error),
    #[error(transparent)]
    Password(InvalidPassword),
    #[error("failed to hash password: {0}")]
    FailedPasswordHash(anyhow::Error),
}

impl From<InvalidUsername> for InvalidRegisterUser {
    fn from(value: InvalidUsername) -> Self {
        Self::Username(value)
    }
}

impl From<email_address::Error> for InvalidRegisterUser {
    fn from(value: email_address::Error) -> Self {
        Self::Email(value)
    }
}

impl From<InvalidPassword> for InvalidRegisterUser {
    fn from(value: InvalidPassword) -> Self {
        Self::Password(value)
    }
}

#[cfg(feature = "zerver")]
/// errors encountered while authenticating a user
#[derive(Debug, Error)]
pub enum AuthenticateUserError {
    #[error("user not found")]
    UserNotFound,
    #[error("invalid password")]
    InvalidPassword,
    #[error(transparent)]
    Database(anyhow::Error),
    #[error("user found but database returned invalid object: {0}")]
    UserFromDb(anyhow::Error),
    #[error("failed to verify password: {0}")]
    FailedToVerify(anyhow::Error),
    #[error("failed to generate `JWT`: {0}")]
    FailedJwt(anyhow::Error),
}

/// errors encountered while constructing `AuthenticateUserRequest`
#[derive(Debug, Error)]
pub enum InvalidAuthenticateUser {
    #[error("identifier must be present")]
    MissingIdentifier,
    #[error(transparent)]
    Password(InvalidPassword),
}

impl From<InvalidPassword> for InvalidAuthenticateUser {
    fn from(value: InvalidPassword) -> Self {
        Self::Password(value)
    }
}

#[cfg(feature = "zerver")]
/// errors encountered while constructing `AuthenticateUserSuccessError`
#[derive(Debug, Error)]
pub enum InvalidAuthenticateUserSuccess {
    #[error(transparent)]
    JwtError(JwtError),
}

/// errors encountered while constructing `ChangePasswordRequestError`
#[derive(Debug, Error)]
pub enum InvalidChangePassword {
    #[error(transparent)]
    Password(InvalidPassword),
    #[error("failed to hash password: {0}")]
    FailedPasswordHash(anyhow::Error),
}

#[cfg(feature = "zerver")]
/// errors encountered while changing password
#[derive(Debug, Error)]
pub enum ChangePasswordError {
    #[error("user not found")]
    UserNotFound,
    #[error(transparent)]
    Database(anyhow::Error),
    #[error(transparent)]
    AuthenticateUserError(AuthenticateUserError),
}

#[cfg(feature = "zerver")]
impl From<AuthenticateUserError> for ChangePasswordError {
    fn from(value: AuthenticateUserError) -> Self {
        Self::AuthenticateUserError(value)
    }
}

#[cfg(feature = "zerver")]
impl From<sqlx::Error> for ChangePasswordError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => Self::UserNotFound,
            e => Self::Database(e.into()),
        }
    }
}

#[derive(Debug, Error)]
pub enum InvalidChangeUsername {
    #[error(transparent)]
    Id(uuid::Error),
    #[error(transparent)]
    Username(InvalidUsername),
}

impl From<uuid::Error> for InvalidChangeUsername {
    fn from(value: uuid::Error) -> Self {
        Self::Id(value)
    }
}

impl From<InvalidUsername> for InvalidChangeUsername {
    fn from(value: InvalidUsername) -> Self {
        Self::Username(value)
    }
}

#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum ChangeUsernameError {
    #[error("user not found")]
    UserNotFound,
    #[error(transparent)]
    Database(anyhow::Error),
    #[error("user updated but database returned invalid object: {0}")]
    UserFromDb(anyhow::Error),
}

#[derive(Debug, Error)]
pub enum InvalidChangeEmail {
    #[error(transparent)]
    Id(uuid::Error),
    #[error(transparent)]
    Email(email_address::Error),
}

impl From<uuid::Error> for InvalidChangeEmail {
    fn from(value: uuid::Error) -> Self {
        Self::Id(value)
    }
}

impl From<email_address::Error> for InvalidChangeEmail {
    fn from(value: email_address::Error) -> Self {
        Self::Email(value)
    }
}

#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum ChangeEmailError {
    #[error("user not found")]
    UserNotFound,
    #[error(transparent)]
    Database(anyhow::Error),
    #[error("user updated but database returned invalid object: {0}")]
    UserFromDb(anyhow::Error),
}

#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum DeleteUserError {
    #[error("user not found")]
    NotFound,
    #[error(transparent)]
    Database(anyhow::Error),
}

// ========================
//  request/response types
// ========================

#[derive(Debug)]
pub struct RawRegisterUser {
    pub username: Username,
    pub email: EmailAddress,
    pub password: Password,
}

impl RawRegisterUser {
    pub fn new(
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<Self, InvalidRawRegisterUser> {
        let username = Username::new(username)?;
        let email = EmailAddress::from_str(email)?;
        let password = Password::new(password)?;
        Ok(Self {
            username,
            email,
            password,
        })
    }
}

#[cfg(feature = "zerver")]
#[derive(Debug)]
pub struct RegisterUser {
    pub username: Username,
    pub email: EmailAddress,
    pub password_hash: HashedPassword,
}

#[cfg(feature = "zerver")]
impl RegisterUser {
    pub fn new(username: &str, email: &str, password: &str) -> Result<Self, InvalidRegisterUser> {
        let username = Username::new(username)?;
        let email = EmailAddress::from_str(email)?;
        let password = Password::new(password)?;
        let password_hash = HashedPassword::generate(password)
            .map_err(|e| InvalidRegisterUser::FailedPasswordHash(e.into()))?;

        Ok(RegisterUser {
            username,
            email,
            password_hash,
        })
    }
}

/// authentication request with identifier (email/username) and password
#[derive(Debug)]
pub struct AuthenticateUser {
    pub identifier: String,
    pub password: String,
}

impl AuthenticateUser {
    pub fn new(identifier: &str, password: &str) -> Result<Self, InvalidAuthenticateUser> {
        if identifier.is_empty() {
            return Err(InvalidAuthenticateUser::MissingIdentifier);
        }
        let password = Password::new(password)?;
        Ok(AuthenticateUser {
            identifier: identifier.to_string(),
            password: password.read().to_string(),
        })
    }
}

#[cfg(feature = "zerver")]
impl From<&ChangePassword> for AuthenticateUser {
    fn from(value: &ChangePassword) -> Self {
        Self {
            identifier: value.user_id.to_string(),
            password: value.current_password.to_owned(),
        }
    }
}

/// successful authentication response containing user data and JWT token
///
/// authentication and register user requeast use this
#[derive(Debug, Serialize, PartialEq, Deserialize)]
pub struct AuthenticateUserSuccess {
    pub user: User,
    pub token: Jwt,
    pub expires_at: usize,
}

#[cfg(feature = "zerver")]
impl AuthenticateUserSuccess {
    pub fn new(
        user: User,
        token_string: String,
        expires_at: usize,
    ) -> Result<Self, InvalidAuthenticateUserSuccess> {
        let token =
            Jwt::new(&token_string).map_err(|e| InvalidAuthenticateUserSuccess::JwtError(e))?;
        Ok(AuthenticateUserSuccess {
            user,
            token,
            expires_at,
        })
    }
}

#[cfg(feature = "zerver")]
/// change password request
/// with idenifier and new password hash
#[derive(Debug)]
pub struct ChangePassword {
    pub user_id: Uuid,
    pub current_password: String,
    pub password_hash: HashedPassword,
}
#[cfg(feature = "zerver")]
impl ChangePassword {
    pub fn new(
        user_id: Uuid,
        current_password: &str,
        new_password: &str,
    ) -> Result<Self, InvalidChangePassword> {
        let password =
            Password::new(new_password).map_err(|e| InvalidChangePassword::Password(e))?;
        let current_password = current_password.to_string();
        let password_hash = HashedPassword::generate(password)
            .map_err(|e| InvalidChangePassword::FailedPasswordHash(e.into()))?;

        Ok(Self {
            user_id,
            current_password,
            password_hash,
        })
    }
}

#[derive(Debug)]
pub struct ChangeUsername {
    pub user_id: Uuid,
    pub username: Username,
}

impl ChangeUsername {
    pub fn new(user_id: Uuid, username: &str) -> Result<Self, InvalidChangeUsername> {
        let username = Username::new(username)?;
        Ok(Self { user_id, username })
    }
}

#[derive(Debug)]
pub struct ChangeEmail {
    pub user_id: Uuid,
    pub email: EmailAddress,
}

impl ChangeEmail {
    pub fn new(user_id: Uuid, email: &str) -> Result<Self, InvalidChangeEmail> {
        let email = EmailAddress::from_str(email)?;
        Ok(Self { user_id, email })
    }
}

#[derive(Debug, Clone)]
pub struct DeleteUser(Uuid);

impl DeleteUser {
    pub fn new(id: &str) -> Result<Self, uuid::Error> {
        let trimmed = id.trim();
        let id = Uuid::try_parse(trimmed)?;
        Ok(Self(id))
    }

    pub fn id(&self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for DeleteUser {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

// ======
//  main
// ======

#[cfg(feature = "zerver")]
/// user entity with password hash
/// for authentication operations
#[derive(Debug)]
pub struct UserWithPasswordHash {
    pub id: Uuid,
    pub username: Username,
    pub email: EmailAddress,
    pub password_hash: HashedPassword,
}

#[cfg(feature = "zerver")]
impl From<UserWithPasswordHash> for User {
    fn from(value: UserWithPasswordHash) -> Self {
        Self {
            id: value.id,
            username: value.username,
            email: value.email,
        }
    }
}
