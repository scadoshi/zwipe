use anyhow::Context;
use email_address::{EmailAddress, Options};
use std::fmt::Display;
use thiserror::Error;

use crate::domain::auth::password::{HashedPassword, Password};
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
