use anyhow::Context;
use email_address::{EmailAddress, Options};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use thiserror::Error;

use crate::domain::auth::models::password::{HashedPassword, Password};
//
//
//
//
//
#[derive(Clone, Debug, Error)]
#[error("Username cannot be empty")]
pub struct UserNameEmptyError;

#[derive(Debug, Error)]
pub enum UserCreationError {
    #[error("User with name or email already exists")]
    Duplicate,
    #[error("Database error: {0}")]
    DatabaseError(anyhow::Error),
    #[error("User created in database but returned an invalid User: {0}")]
    ReturnedUserInvalid(anyhow::Error),
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("User ID must be between 0 and 999,999")]
pub struct UserIdError;
//
//
//
//
//
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UserName(String);

impl UserName {
    pub fn new(raw: &str) -> Result<Self, UserNameEmptyError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            Err(UserNameEmptyError)
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

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UserId(i32);

impl UserId {
    pub fn new(raw: i32) -> Result<Self, UserIdError> {
        if raw < 0 || raw > 999_999 {
            return Err(UserIdError);
        }
        Ok(Self(raw))
    }
}

/// ensures we use our newtype validation logic in its constructor
/// while deserializing
impl<'de> Deserialize<'de> for UserId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = i32::deserialize(deserializer)?;
        UserId::new(raw).map_err(serde::de::Error::custom)
    }
}
/// ensuire we send out only the contents of our newtype
/// wrapper struct as we don't want to send
/// out a wrapper integer as that wouldn't be expected
/// by external APIs
impl Serialize for UserId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}
//
//
//
//
//

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct UserCreationRequest {
    pub email: EmailAddress,
    pub username: UserName,
    pub password_hash: HashedPassword,
}

impl UserCreationRequest {
    pub fn new(username: &str, email: &str, password: &str) -> Result<Self, anyhow::Error> {
        let username = UserName::new(username).context("Invalid username")?;
        let email = EmailAddress::parse_with_options(email, Options::default())
            .context("Invalid email address")?;
        let password_hash =
            HashedPassword::new(Password::new(password).context("Invalid password")?)
                .context("Failed to hash password")?;
        Ok(UserCreationRequest {
            email,
            username,
            password_hash,
        })
    }
}
//
//
//
//
//
#[derive(Debug, Clone)]
pub struct User {
    pub id: UserId,
    pub email: EmailAddress,
    pub username: UserName,
}
