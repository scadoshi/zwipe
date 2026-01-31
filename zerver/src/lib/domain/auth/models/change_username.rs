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
    Id(#[from] uuid::Error),
    #[error(transparent)]
    Username(#[from] InvalidUsername),
    #[error(transparent)]
    Password(#[from] InvalidPassword),
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
    AuthenticateUserError(#[from] AuthenticateUserError),
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
