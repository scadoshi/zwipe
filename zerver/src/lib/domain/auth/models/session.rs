use crate::domain::auth::models::access_token::AccessToken;
use crate::domain::auth::models::refresh_token::RefreshToken;
use crate::domain::user::models::User;
#[cfg(feature = "zerver")]
use crate::domain::{auth::models::access_token::InvalidJwt, user::models::get_user::GetUserError};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;
use uuid::Uuid;

pub const MAXIMUM_SESSION_COUNT: u8 = 5;

// ========
//  errors
// ========

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

#[derive(Debug, Error)]
pub enum InvalidRefreshSession {
    #[error(transparent)]
    UserId(uuid::Error),
}

#[cfg(feature = "zerver")]
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

#[derive(Debug, Error)]
pub enum InvalidRevokeSessions {
    #[error(transparent)]
    UserId(uuid::Error),
}

impl From<uuid::Error> for InvalidRevokeSessions {
    fn from(value: uuid::Error) -> Self {
        Self::UserId(value)
    }
}

#[derive(Debug, Error)]
pub enum RevokeSessionsError {
    #[error(transparent)]
    Database(anyhow::Error),
}

#[derive(Debug, Error)]
pub enum EnforceSessionMaximumError {
    #[error(transparent)]
    Database(anyhow::Error),
}

#[derive(Debug, Error)]
pub enum DeleteExpiredSessionsError {
    #[error(transparent)]
    Database(anyhow::Error),
}

// =========
//  request
// =========

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

#[derive(Debug, Clone)]
pub struct RefreshSession {
    pub user_id: Uuid,
    pub refresh_token: String,
}

#[cfg(feature = "zerver")]
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

#[derive(Debug, Clone)]
pub struct RevokeSessions {
    pub user_id: Uuid,
}

impl RevokeSessions {
    pub fn new(user_id: Uuid) -> Self {
        Self { user_id }
    }
}

impl FromStr for RevokeSessions {
    type Err = InvalidRevokeSessions;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let user_id = Uuid::try_parse(s)?;
        Ok(Self { user_id })
    }
}

impl From<Uuid> for RevokeSessions {
    fn from(value: Uuid) -> Self {
        Self::new(value)
    }
}

// ======
//  main
// ======

/// successful authentication response containing user data and tokens
#[derive(Debug, Serialize, PartialEq, Deserialize)]
pub struct Session {
    pub user: User,
    pub access_token: AccessToken,
    pub refresh_token: RefreshToken,
}

#[cfg(feature = "zerver")]
impl Session {
    pub fn new(user: User, access_token: AccessToken, refresh_token: RefreshToken) -> Self {
        Session {
            user,
            access_token,
            refresh_token,
        }
    }
}
