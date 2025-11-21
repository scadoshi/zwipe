use crate::domain::auth::models::session::Session;
#[cfg(feature = "zerver")]
use crate::domain::{
    auth::models::{
        access_token::InvalidJwt,
        session::{
            create_session::CreateSessionError, enforce_session_maximum::EnforceSessionMaximumError,
        },
    },
    user::models::get_user::GetUserError,
};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum InvalidRefreshSession {
    #[error(transparent)]
    UserId(uuid::Error),
}

impl From<uuid::Error> for InvalidRefreshSession {
    fn from(value: uuid::Error) -> Self {
        Self::UserId(value)
    }
}

#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum RefreshSessionError {
    #[error(transparent)]
    Database(anyhow::Error),
    #[error(transparent)]
    CreateSessionError(CreateSessionError),
    #[error("match for given refresh token not found—user attempting: {0}")]
    NotFound(Uuid),
    #[error("given refresh token is expired—user attempting: {0}")]
    Expired(Uuid),
    #[error("given refresh token has been revoked—user attempting: {0}")]
    Revoked(Uuid),
    #[error("refresh token does not belong to the requesting user—user attempting: {0}")]
    Forbidden(Uuid),
    #[error(transparent)]
    GetUserError(GetUserError),
    #[error(transparent)]
    InvalidJwt(InvalidJwt),
    #[error(transparent)]
    EnforceSessionMaximumError(EnforceSessionMaximumError),
}

#[cfg(feature = "zerver")]
impl From<EnforceSessionMaximumError> for RefreshSessionError {
    fn from(value: EnforceSessionMaximumError) -> Self {
        Self::EnforceSessionMaximumError(value)
    }
}

#[cfg(feature = "zerver")]
impl From<InvalidJwt> for RefreshSessionError {
    fn from(value: InvalidJwt) -> Self {
        Self::InvalidJwt(value)
    }
}

#[cfg(feature = "zerver")]
impl From<GetUserError> for RefreshSessionError {
    fn from(value: GetUserError) -> Self {
        Self::GetUserError(value)
    }
}

#[cfg(feature = "zerver")]
impl From<CreateSessionError> for RefreshSessionError {
    fn from(value: CreateSessionError) -> Self {
        Self::CreateSessionError(value)
    }
}

#[derive(Debug, Clone)]
pub struct RefreshSession {
    pub user_id: Uuid,
    pub refresh_token: String,
}

impl RefreshSession {
    pub fn new(user_id: &str, refresh_token: &str) -> Result<Self, InvalidRefreshSession> {
        let user_id = Uuid::try_parse(user_id)?;
        let refresh_token = refresh_token.to_string();
        Ok(Self {
            user_id,
            refresh_token,
        })
    }
}

impl From<&Session> for RefreshSession {
    fn from(value: &Session) -> Self {
        Self {
            user_id: value.user.id,
            refresh_token: value.refresh_token.value.clone(),
        }
    }
}
