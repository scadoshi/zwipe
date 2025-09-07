use email_address::EmailAddress;
use serde::Serialize;
use std::fmt::Display;
use thiserror::Error;
use uuid::Uuid;

// ========
//  errors
// ========

#[derive(Debug, Error, Clone)]
pub enum UsernameError {
    #[error("length must be greater than 0")]
    TooShort,
    #[error("length must be less than 21")]
    TooLong,
}

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
    pub fn new(raw: &str) -> Result<Self, UsernameError> {
        let trimmed = raw.trim();

        if trimmed.is_empty() {
            return Err(UsernameError::TooShort);
        }

        if trimmed.len() > 20 {
            return Err(UsernameError::TooLong);
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
