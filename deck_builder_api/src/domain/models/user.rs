use chrono::NaiveDateTime;
use email_address::{EmailAddress, Options};
use serde::{Deserialize, Serialize};
use sqlx_macros::FromRow;
use std::fmt::Display;
use thiserror::Error;
//
//
//
//
//
#[derive(Clone, Debug, Error)]
#[error("Username cannot be empty")]
pub struct UserNameEmptyError;

#[derive(Debug, Error)]
pub enum UserCreationRequestError {
    #[error(transparent)]
    InvalidUsername(#[from] UserNameEmptyError),
    #[error(transparent)]
    InvalidEmail(#[from] email_address::Error),
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum UserCreationError {
    #[error("User with name {name} or email {email} already exists")]
    Duplicate { name: UserName, email: EmailAddress },
    // continue with more errors as we go
}
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
//
//
//
//
//

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserCreationRequest {
    pub email: EmailAddress,
    pub username: UserName,
}

impl UserCreationRequest {
    fn new(username: &str, email: &str) -> Result<Self, UserCreationRequestError> {
        let username = UserName::new(username)?;
        let email = EmailAddress::parse_with_options(email, Options::default())?;
        Ok(UserCreationRequest { email, username })
    }
}
//
//
//
//
//
#[derive(Debug, Clone, Deserialize, Serialize, FromRow)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
