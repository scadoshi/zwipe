use crate::domain::auth::models::access_token::AccessToken;
use crate::domain::auth::models::refresh_token::RefreshToken;
#[cfg(feature = "zerver")]
use crate::domain::auth::models::{
    access_token::InvalidAccessToken, refresh_token::InvalidRefreshToken,
};
use crate::domain::user::models::User;
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
}

#[cfg(feature = "zerver")]
impl From<EnforceSessionMaximumError> for CreateSessionError {
    fn from(value: EnforceSessionMaximumError) -> Self {
        Self::EnforceSessionMaximumError(value)
    }
}

#[cfg(feature = "zerver")]
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
    #[error("match for given refresh token not found")]
    NotFound,
    #[error("given refresh token is expired")]
    Expired,
    #[error("given refresh token has been revoked")]
    Revoked,
    #[error("refresh token does not belong to the requesting user")]
    Forbidden,
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
#[derive(Debug, Clone)]
pub struct CreateSession(Uuid);

#[cfg(feature = "zerver")]
impl CreateSession {
    pub fn user_id(&self) -> &Uuid {
        &self.0
    }
}

#[cfg(feature = "zerver")]
impl FromStr for CreateSession {
    type Err = InvalidCreateSession;
    fn from_str(s: &str) -> Result<Self, InvalidCreateSession> {
        Ok(Self(Uuid::try_parse(s)?))
    }
}

#[cfg(feature = "zerver")]
impl From<&RefreshSession> for CreateSession {
    fn from(value: &RefreshSession) -> Self {
        Self(value.user_id.to_owned())
    }
}

#[cfg(feature = "zerver")]
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
pub struct RevokeSessions(Uuid);

impl RevokeSessions {
    pub fn user_id(&self) -> &Uuid {
        &self.0
    }
}

impl FromStr for RevokeSessions {
    type Err = InvalidRevokeSessions;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let user_id = Uuid::try_parse(s)?;
        Ok(Self(user_id))
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
