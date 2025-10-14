#[cfg(feature = "zerver")]
use std::str::FromStr;

#[cfg(feature = "zerver")]
use crate::domain::{
    auth::models::{
        access_token::InvalidJwt, session::enforce_session_maximum::EnforceSessionMaximumError,
    },
    user::models::get_user::GetUserError,
};
#[cfg(feature = "zerver")]
use thiserror::Error;
#[cfg(feature = "zerver")]
use uuid::Uuid;

#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum InvalidCreateSession {
    #[error(transparent)]
    UserId(uuid::Error),
}

#[cfg(feature = "zerver")]
impl From<uuid::Error> for InvalidCreateSession {
    fn from(value: uuid::Error) -> Self {
        Self::UserId(value)
    }
}

#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum CreateSessionError {
    #[error(transparent)]
    Database(anyhow::Error),
    #[error(transparent)]
    EnforceSessionMaximumError(EnforceSessionMaximumError),
    #[error(transparent)]
    GetUserError(GetUserError),
    #[error(transparent)]
    InvalidJwt(InvalidJwt),
}

#[cfg(feature = "zerver")]
impl From<InvalidJwt> for CreateSessionError {
    fn from(value: InvalidJwt) -> Self {
        Self::InvalidJwt(value)
    }
}

#[cfg(feature = "zerver")]
impl From<GetUserError> for CreateSessionError {
    fn from(value: GetUserError) -> Self {
        Self::GetUserError(value)
    }
}

#[cfg(feature = "zerver")]
impl From<EnforceSessionMaximumError> for CreateSessionError {
    fn from(value: EnforceSessionMaximumError) -> Self {
        Self::EnforceSessionMaximumError(value)
    }
}

#[cfg(feature = "zerver")]
impl FromStr for CreateSession {
    type Err = InvalidCreateSession;
    fn from_str(s: &str) -> Result<Self, InvalidCreateSession> {
        Ok(Self {
            user_id: Uuid::try_parse(s)?,
        })
    }
}

#[cfg(feature = "zerver")]
#[derive(Debug, Clone)]
pub struct CreateSession {
    pub user_id: Uuid,
}

#[cfg(feature = "zerver")]
impl From<Uuid> for CreateSession {
    fn from(value: Uuid) -> Self {
        Self { user_id: value }
    }
}
