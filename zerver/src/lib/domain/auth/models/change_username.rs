use crate::domain::user::models::{InvalidUsername, Username};
use thiserror::Error;
use uuid::Uuid;

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
