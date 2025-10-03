use email_address::EmailAddress;
use serde::Serialize;
use std::fmt::Display;
use thiserror::Error;
use uuid::Uuid;

use crate::domain::auth::models::bad_words::ContainsBadWord;

// ========
//  errors
// ========

#[derive(Debug, Error, Clone)]
pub enum InvalidUsername {
    #[error("must be at least 3 characters long")]
    TooShort,
    #[error("must not exceed 20 characters")]
    TooLong,
    #[error("cannot contain whitespace")]
    Whitespace,
    #[error("no naughty bad words please")]
    BadWord,
}

#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum GetUserError {
    #[error("user not found")]
    NotFound,
    #[error(transparent)]
    Database(anyhow::Error),
    #[error("user found but database returned invalid object: {0}")]
    UserFromDb(anyhow::Error),
}

// ==========
//  newtypes
// ==========

#[derive(Clone, Debug, PartialEq)]
pub struct Username(String);

impl Username {
    pub fn new(raw: &str) -> Result<Self, InvalidUsername> {
        let trimmed = raw.trim();

        if trimmed.contains_bad_word() {
            return Err(InvalidUsername::BadWord);
        }

        if trimmed.chars().any(|c| c.is_whitespace()) {
            return Err(InvalidUsername::Whitespace);
        }

        if trimmed.len() < 3 {
            return Err(InvalidUsername::TooShort);
        }

        if trimmed.len() > 20 {
            return Err(InvalidUsername::TooLong);
        }

        Ok(Self(trimmed.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for Username {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for Username {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

// ==========
//  requests
// ==========

#[derive(Debug, Clone)]
pub struct GetUser(Uuid);

impl GetUser {
    pub fn new(id: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::try_parse(id)?))
    }

    pub fn id(&self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for GetUser {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

// ======
//  main
// ======

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct User {
    pub id: Uuid,
    pub username: Username,
    pub email: EmailAddress,
}
