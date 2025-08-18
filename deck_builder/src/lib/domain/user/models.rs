use anyhow::Context;
use email_address::{EmailAddress, Options};
use serde::{Deserialize, Serialize};
use std::{f64::consts::E, fmt::Display, str::FromStr};
use thiserror::Error;
use uuid::Uuid;

use crate::domain::auth::models::{
    jwt::{Jwt, JwtError},
    password::{HashedPassword, Password},
};

// =============================================================================
// ERRORS
// =============================================================================

#[derive(Clone, Debug, Error)]
pub enum UserNameError {
    #[error("Username must be present")]
    MissingUserName,
}

// #[derive(Debug, Clone, PartialEq, Eq, Error)]
// #[error("User ID must be between 0 and 999,999")]
// pub struct UserIdError;

/// For constructor of UserRequest
#[derive(Debug, Clone, Error)]
pub enum UserRequestError {
    #[error(transparent)]
    InvalidUsername(UserNameEmptyError),
    #[error(transparent)]
    InvalidEmail(email_address::Error),
}

/// Actual errors encountered while creating or updating a user
#[derive(Debug, Error)]
pub enum UserError {
    #[error("User with name or email already exists")]
    Duplicate,
    #[error("Database issues: {0}")]
    DatabaseIssues(anyhow::Error),
    #[error("User created but database returned invalid User. DatabaseUser -> User conversion error: {0}")]
    InvalidUserFromDatabase(anyhow::Error),
}

/// Actual errors encountered while getting a user
#[derive(Debug, Clone, Error)]
pub enum GetUserError {
    #[error("User not found")]
    NotFound,
}

#[derive(Clone, Debug, Error)]
pub enum DeleteUserRequestError {
    #[error("Id must be present")]
    MissingId,
    #[error("Failed to parse Uuid: {0}")]
    FailedUuid(uuid::Error),
}

#[derive(Debug, Clone, Error)]
pub enum DeleteUserError {
    #[error("User not found")]
    NotFound,
}

// =============================================================================
// NEWTYPES
// =============================================================================

// since i am using Uuid for everything now
// i don't think i need this commenting out for now
// i may be back
//
// /// Validated user ID within range 0-999,999
// #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Copy)]
// pub struct UserId(i32);

// impl UserId {
//     pub fn new(raw: i32) -> Result<Self, UserIdError> {
//         if raw < 0 || raw > 999_999 {
//             return Err(UserIdError);
//         }
//         Ok(Self(raw))
//     }
// }

// impl<'de> Deserialize<'de> for UserId {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         let raw = i32::deserialize(deserializer)?;
//         UserId::new(raw).map_err(serde::de::Error::custom)
//     }
// }

// impl Serialize for UserId {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         self.0.serialize(serializer)
//     }
// }

/// Validated username that cannot be empty or whitespace-only
#[derive(Clone, Debug)]
pub struct UserName(String);

impl UserName {
    pub fn new(raw: &str) -> Result<Self, UserNameError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            Err(UserNameError::MissingUserName)
        } else {
            Ok(Self(trimmed.to_string()))
        }
    }
}

impl Display for UserName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// =============================================================================
// REQUESTS
// =============================================================================

#[derive(Debug, Clone)]
pub struct CreateUserRequest {
    pub email: EmailAddress,
    pub username: UserName,
}

impl CreateUserRequest {
    pub fn new(
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<Self, CreateUserRequestError> {
        let username = UserName::new(username).map_err(|e| UserRequestError::InvalidUsername(e))?;
        let email = EmailAddress::from_str(email).map_err(|e| UserRequestError::InvalidEmail(e))?;
        Ok(CreateUserRequest { email, username })
    }
}

#[derive(Debug, Clone)]
pub struct GetUserRequest {
    pub identifier: String,
}

impl GetUserRequest {
    pub fn new(identifier: &str) -> Self {
        let identifier = identifier.to_string();
        GetUserRequest { identifier }
    }
}

#[derive(Debug, Clone)]
pub struct UpdateUserRequest {
    pub username: Option<UserName>,
    pub email: Option<EmailAddress>,
}

impl UpdateUserRequest {
    fn new(username_opt: Option<&str>, email_opt: Option<&str>) -> Result<Self, UserRequestError> {
        let username = username_opt
            .map(|username_str| {
                UserName::new(username_str).map_err(|e| UserRequestError::InvalidUsername(e))
            })
            .transpose()?;

        let email = email_opt
            .map(|email_str| {
                EmailAddress::from_str(email_str).map_err(|e| UserRequestError::InvalidEmail(e))
            })
            .transpose()?;

        Ok(Self { username, email })
    }
}

#[derive(Debug, Clone)]
pub struct DeleteUserRequest(Uuid);

impl DeleteUserRequest {
    fn new(id: &str) -> Result<Self, DeleteUserRequestError> {
        let trimmed = id.trim();
        if trimmed.is_empty() {
            return Err(DeleteUserRequestError::MissingId);
        }

        let id = Uuid::try_parse(trimmed).map_err(|e| DeleteUserRequestError::FailedUuid(e))?;
        Ok(Self(id))
    }
}

// =============================================================================
// MAINS
// =============================================================================

#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub username: UserName,
    pub email: EmailAddress,
}
