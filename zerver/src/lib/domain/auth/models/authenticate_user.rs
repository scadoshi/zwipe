use crate::domain::auth::models::password::{InvalidPassword, Password};
#[cfg(feature = "zerver")]
use crate::domain::auth::models::{
    change_password::ChangePassword, session::create_session::CreateSessionError,
};
use thiserror::Error;

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
    #[error("failed to generate access token: {0}")]
    FailedAccessToken(anyhow::Error),
    #[error(transparent)]
    CreateSessionError(CreateSessionError),
}

#[cfg(feature = "zerver")]
impl From<CreateSessionError> for AuthenticateUserError {
    fn from(value: CreateSessionError) -> Self {
        Self::CreateSessionError(value)
    }
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
