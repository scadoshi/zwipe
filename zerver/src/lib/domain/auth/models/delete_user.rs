#[cfg(feature = "zerver")]
use crate::domain::auth::models::authenticate_user::AuthenticateUserError;
use thiserror::Error;
use uuid::Uuid;

#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum DeleteUserError {
    #[error("user not found")]
    NotFound,
    #[error(transparent)]
    Database(anyhow::Error),
    #[error(transparent)]
    AuthenticateUserError(#[from] AuthenticateUserError),
}

#[derive(Debug, Error)]
pub enum InvalidDeleteUser {
    #[error(transparent)]
    Userid(uuid::Error),
    #[error("invalid password")]
    Password,
}

impl From<uuid::Error> for InvalidDeleteUser {
    fn from(value: uuid::Error) -> Self {
        Self::Userid(value)
    }
}

#[derive(Debug, Clone)]
pub struct DeleteUser {
    pub user_id: Uuid,
    pub password: String,
}

impl DeleteUser {
    pub fn new(user_id: Uuid, password: &str) -> Result<Self, InvalidDeleteUser> {
        let password = password.trim();
        Ok(Self {
            user_id,
            password: password.to_string(),
        })
    }
}
