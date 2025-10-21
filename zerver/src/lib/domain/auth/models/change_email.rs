use email_address::EmailAddress;
use std::str::FromStr;
use thiserror::Error;
use uuid::Uuid;

#[cfg(feature = "zerver")]
use crate::domain::auth::models::authenticate_user::AuthenticateUserError;
use crate::domain::auth::models::password::{InvalidPassword, Password};

#[derive(Debug, Error)]
pub enum InvalidChangeEmail {
    #[error(transparent)]
    Id(uuid::Error),
    #[error(transparent)]
    Email(email_address::Error),
    #[error(transparent)]
    Password(InvalidPassword),
}

impl From<InvalidPassword> for InvalidChangeEmail {
    fn from(value: InvalidPassword) -> Self {
        Self::Password(value)
    }
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
    #[error(transparent)]
    AuthenticateUserError(AuthenticateUserError),
    #[error("user with that email already exists")]
    Duplicate,
}

#[cfg(feature = "zerver")]
impl From<AuthenticateUserError> for ChangeEmailError {
    fn from(value: AuthenticateUserError) -> Self {
        Self::AuthenticateUserError(value)
    }
}

#[derive(Debug)]
pub struct ChangeEmail {
    pub user_id: Uuid,
    pub email: EmailAddress,
    pub password: Password,
}

impl ChangeEmail {
    pub fn new(user_id: Uuid, email: &str, password: &str) -> Result<Self, InvalidChangeEmail> {
        let email = EmailAddress::from_str(email)?;
        let password = Password::new(password)?;
        Ok(Self {
            user_id,
            email,
            password,
        })
    }
}
