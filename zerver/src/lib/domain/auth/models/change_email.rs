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
    Id(#[from] uuid::Error),
    #[error(transparent)]
    Email(#[from] email_address::Error),
    #[error(transparent)]
    Password(#[from] InvalidPassword),
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
    AuthenticateUserError(#[from] AuthenticateUserError),
    #[error("user with that email already exists")]
    Duplicate,
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
