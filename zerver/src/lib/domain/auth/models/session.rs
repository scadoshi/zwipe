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
    #[error("add error enumerations here as you go")]
    Todo,
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
    #[error("add error enumerations here as you go")]
    Todo,
}

#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum InvalidSession {
    #[error(transparent)]
    InvalidAccessToken(InvalidAccessToken),
    #[error(transparent)]
    InvalidRefreshToken(InvalidRefreshToken),
}

#[cfg(feature = "zerver")]
impl From<InvalidAccessToken> for InvalidSession {
    fn from(value: InvalidAccessToken) -> Self {
        Self::InvalidAccessToken(value)
    }
}

#[cfg(feature = "zerver")]
impl From<InvalidRefreshToken> for InvalidSession {
    fn from(value: InvalidRefreshToken) -> Self {
        Self::InvalidRefreshToken(value)
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
#[derive(Debug, Clone)]
pub struct RefreshSession {
    user_id: Uuid,
    refresh_token: String,
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

#[derive(Debug, Error)]
pub enum RevokeSessionsError {
    #[error("add error enumerations here")]
    Todo,
}

// ======
//  main
// ======

/// successful authentication response containing user data and tokens
#[derive(Debug, Serialize, PartialEq, Deserialize)]
pub struct Session {
    pub user: User,
    pub access_token: AccessToken,
    pub access_token_expires_at: usize,
    pub refresh_token: RefreshToken,
    pub refresh_token_expires_at: usize,
}

#[cfg(feature = "zerver")]
impl Session {
    pub fn new(
        user: User,
        access_token_string: String,
        access_token_expires_at: usize,
        refresh_token_string: String,
        refresh_token_expires_at: usize,
    ) -> Result<Self, InvalidSession> {
        let access_token = AccessToken::from_str(&access_token_string)?;
        let refresh_token = RefreshToken::from_str(&refresh_token_string)?;
        Ok(Session {
            user,
            access_token,
            access_token_expires_at,
            refresh_token,
            refresh_token_expires_at,
        })
    }
}
