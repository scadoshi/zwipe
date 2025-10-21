#[cfg(feature = "zerver")]
use crate::domain::auth::models::authenticate_user::AuthenticateUserError;
use crate::domain::{
    auth::models::password::{InvalidPassword, Password},
    user::models::username::{InvalidUsername, Username},
};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum InvalidChangeUsername {
    #[error(transparent)]
    Id(uuid::Error),
    #[error(transparent)]
    Username(InvalidUsername),
    #[error(transparent)]
    Password(InvalidPassword),
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

impl From<InvalidPassword> for InvalidChangeUsername {
    fn from(value: InvalidPassword) -> Self {
        Self::Password(value)
    }
}

#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum ChangeUsernameError {
    #[error("user not found")]
    UserNotFound,
    #[error(transparent)]
    Database(anyhow::Error),
    #[error("user with that username already exists")]
    Duplicate,
    #[error("database returned invalid object: {0}")]
    UserFromDb(anyhow::Error),
    #[error(transparent)]
    AuthenticateUserError(AuthenticateUserError),
}

#[cfg(feature = "zerver")]
impl From<AuthenticateUserError> for ChangeUsernameError {
    fn from(value: AuthenticateUserError) -> Self {
        Self::AuthenticateUserError(value)
    }
}

#[derive(Debug)]
pub struct ChangeUsername {
    pub user_id: Uuid,
    pub new_username: Username,
    pub password: Password,
}

impl ChangeUsername {
    pub fn new(
        user_id: Uuid,
        new_username: &str,
        password: &str,
    ) -> Result<Self, InvalidChangeUsername> {
        let new_username = Username::new(new_username)?;
        let password = Password::new(password)?;
        Ok(Self {
            user_id,
            new_username,
            password,
        })
    }
}
