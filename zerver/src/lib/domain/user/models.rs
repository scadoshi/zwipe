use email_address::EmailAddress;
use serde::Serialize;
use std::{fmt::Display, str::FromStr};
use thiserror::Error;
use uuid::Uuid;

use crate::domain::DatabaseError;

// ========
//  errors
// ========

#[derive(Debug, Error, Clone)]
pub enum UserNameError {
    #[error("username must be present")]
    MissingUserName,
}

/// for constructor of `CreateUserRequest`
#[derive(Debug, Error, Clone)]
pub enum CreateUserRequestError {
    #[error(transparent)]
    InvalidUsername(UserNameError),
    #[error(transparent)]
    InvalidEmail(email_address::Error),
}

impl From<UserNameError> for CreateUserRequestError {
    fn from(value: UserNameError) -> Self {
        CreateUserRequestError::InvalidUsername(value)
    }
}

impl From<email_address::Error> for CreateUserRequestError {
    fn from(value: email_address::Error) -> Self {
        CreateUserRequestError::InvalidEmail(value)
    }
}

/// actual errors encountered while creating a user
#[derive(Debug, Error)]
pub enum CreateUserError {
    #[error("user with name or email already exists")]
    Duplicate,
    #[error("user created but database returned invalid object: {0}")]
    InvalidUserFromDatabase(anyhow::Error),
    #[error(transparent)]
    Database(DatabaseError),
}

/// actual errors encountered while getting a user
#[derive(Debug, Error)]
pub enum GetUserError {
    #[error("user not found")]
    NotFound,
    #[error(transparent)]
    Database(DatabaseError),
    #[error("user found but database returned invalid object: {0}")]
    InvalidUserFromDatabase(anyhow::Error),
}

/// for constructor of `UpdateUserRequest`
#[derive(Debug, Error)]
pub enum UpdateUserRequestError {
    #[error(transparent)]
    InvalidId(uuid::Error),
    #[error("must update at least one field")]
    NothingToUpdate,
    #[error(transparent)]
    InvalidUsername(UserNameError),
    #[error(transparent)]
    InvalidEmail(email_address::Error),
}

impl From<uuid::Error> for UpdateUserRequestError {
    fn from(value: uuid::Error) -> Self {
        UpdateUserRequestError::InvalidId(value)
    }
}

impl From<UserNameError> for UpdateUserRequestError {
    fn from(value: UserNameError) -> Self {
        UpdateUserRequestError::InvalidUsername(value)
    }
}

impl From<email_address::Error> for UpdateUserRequestError {
    fn from(value: email_address::Error) -> Self {
        UpdateUserRequestError::InvalidEmail(value)
    }
}

/// actual errors encountered while updating a user
#[derive(Debug, Error)]
pub enum UpdateUserError {
    #[error("user with name or email already exists")]
    Duplicate,
    #[error("user not found")]
    NotFound,
    #[error(transparent)]
    Database(DatabaseError),
    #[error("user updated but database returned invalid object: {0}")]
    InvalidUserFromDatabase(anyhow::Error),
}

/// actual errors encountered while deleting a user
#[derive(Debug, Error)]
pub enum DeleteUserError {
    #[error("user not found")]
    NotFound,
    #[error(transparent)]
    Database(DatabaseError),
}

// ==========
//  newtypes
// ==========

/// validated username that cannot be empty or whitespace-only
#[derive(Clone, Debug, PartialEq)]
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

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for UserName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for UserName {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

// ==========
//  requests
// ==========

#[derive(Debug, Clone)]
pub struct CreateUserRequest {
    pub username: UserName,
    pub email: EmailAddress,
}

impl CreateUserRequest {
    pub fn new(username: &str, email: &str) -> Result<Self, CreateUserRequestError> {
        let username = UserName::new(username)?;
        let email = EmailAddress::from_str(email)?;
        Ok(Self { email, username })
    }
}

#[derive(Debug, Clone)]
pub struct GetUserRequest(String);

impl GetUserRequest {
    pub fn new(identifier: &str) -> Self {
        Self(identifier.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct UpdateUserRequest {
    pub id: Uuid,
    pub username: Option<UserName>,
    pub email: Option<EmailAddress>,
}

impl UpdateUserRequest {
    pub fn new(
        id: &str,
        username: Option<String>,
        email: Option<String>,
    ) -> Result<Self, UpdateUserRequestError> {
        if username.is_none() && email.is_none() {
            return Err(UpdateUserRequestError::NothingToUpdate);
        }

        let id = Uuid::try_parse(id)?;
        let username = username
            .map(|username_str| UserName::new(&username_str))
            .transpose()?;
        let email = email
            .map(|email_str| EmailAddress::from_str(&email_str))
            .transpose()?;

        Ok(Self {
            id,
            username,
            email,
        })
    }
}

#[derive(Debug, Clone)]
pub struct DeleteUserRequest(Uuid);

impl DeleteUserRequest {
    pub fn new(id: &str) -> Result<Self, uuid::Error> {
        let trimmed = id.trim();
        let id = Uuid::try_parse(trimmed)?;
        Ok(Self(id))
    }

    pub fn id(&self) -> Uuid {
        self.0
    }
}

// ======
//  main
// ======

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct User {
    pub id: Uuid,
    pub username: UserName,
    pub email: EmailAddress,
}
