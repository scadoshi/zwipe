#[cfg(feature = "zerver")]
use crate::domain::auth::models::password::HashedPassword;
#[cfg(feature = "zerver")]
use crate::domain::auth::models::session::CreateSessionError;
use crate::domain::user::models::username::Username;
use crate::domain::{
    auth::models::password::{InvalidPassword, Password},
    user::models::username::InvalidUsername,
};
use email_address::EmailAddress;
use std::str::FromStr;
use thiserror::Error;

#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum RegisterUserError {
    #[error("user with name or email already exists")]
    Duplicate,
    #[error(transparent)]
    Database(anyhow::Error),
    #[error("failed to generate access token: {0}")]
    FailedAccessToken(anyhow::Error),
    #[error("user created but database returned invalid object: {0}")]
    UserFromDb(anyhow::Error),
    #[error(transparent)]
    CreateSessionError(CreateSessionError),
}

#[cfg(feature = "zerver")]
impl From<CreateSessionError> for RegisterUserError {
    fn from(value: CreateSessionError) -> Self {
        Self::CreateSessionError(value)
    }
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
        use std::str::FromStr;

        use crate::domain::auth::models::password::Password;

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
