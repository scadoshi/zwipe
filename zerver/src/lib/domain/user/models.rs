use email_address::EmailAddress;
use serde::Serialize;
use std::{fmt::Display, str::FromStr};
use thiserror::Error;
use uuid::Uuid;

// ========
//  errors
// ========

#[derive(Debug, Error, Clone)]
pub enum UserNameError {
    #[error("username must be present")]
    NotFound,
}

/// for constructor of `CreateUser`
#[derive(Debug, Error, Clone)]
pub enum InvalidCreateUser {
    #[error(transparent)]
    Username(UserNameError),
    #[error(transparent)]
    Email(email_address::Error),
}

impl From<UserNameError> for InvalidCreateUser {
    fn from(value: UserNameError) -> Self {
        InvalidCreateUser::Username(value)
    }
}

impl From<email_address::Error> for InvalidCreateUser {
    fn from(value: email_address::Error) -> Self {
        InvalidCreateUser::Email(value)
    }
}

/// actual errors encountered while creating a user
#[derive(Debug, Error)]
pub enum CreateUserError {
    #[error("user with name or email already exists")]
    Duplicate,
    #[error("user created but database returned invalid object: {0}")]
    UserFromDb(anyhow::Error),
    #[error(transparent)]
    Database(anyhow::Error),
}

/// actual errors encountered while getting a user
#[derive(Debug, Error)]
pub enum GetUserError {
    #[error("user not found")]
    NotFound,
    #[error(transparent)]
    Database(anyhow::Error),
    #[error("user found but database returned invalid object: {0}")]
    UserFromDb(anyhow::Error),
}

/// for constructor of `UpdateUser`
#[derive(Debug, Error)]
pub enum InvalidUpdateUser {
    #[error(transparent)]
    Id(uuid::Error),
    #[error("must update at least one field")]
    NoUpdates,
    #[error(transparent)]
    Username(UserNameError),
    #[error(transparent)]
    Email(email_address::Error),
}

impl From<uuid::Error> for InvalidUpdateUser {
    fn from(value: uuid::Error) -> Self {
        InvalidUpdateUser::Id(value)
    }
}

impl From<UserNameError> for InvalidUpdateUser {
    fn from(value: UserNameError) -> Self {
        InvalidUpdateUser::Username(value)
    }
}

impl From<email_address::Error> for InvalidUpdateUser {
    fn from(value: email_address::Error) -> Self {
        InvalidUpdateUser::Email(value)
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
    Database(anyhow::Error),
    #[error("user updated but database returned invalid object: {0}")]
    UserFromDb(anyhow::Error),
}

/// actual errors encountered while deleting a user
#[derive(Debug, Error)]
pub enum DeleteUserError {
    #[error("user not found")]
    NotFound,
    #[error(transparent)]
    Database(anyhow::Error),
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
            Err(UserNameError::NotFound)
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
pub struct CreateUser {
    pub username: UserName,
    pub email: EmailAddress,
}

impl CreateUser {
    pub fn new(username: &str, email: &str) -> Result<Self, InvalidCreateUser> {
        let username = UserName::new(username)?;
        let email = EmailAddress::from_str(email)?;
        Ok(Self { email, username })
    }
}

#[derive(Debug, Clone)]
pub struct GetUser(String);

impl GetUser {
    pub fn new(identifier: &str) -> Self {
        Self(identifier.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct UpdateUser {
    pub id: Uuid,
    pub username: Option<UserName>,
    pub email: Option<EmailAddress>,
}

impl UpdateUser {
    pub fn new(
        id: &str,
        username: Option<String>,
        email: Option<String>,
    ) -> Result<Self, InvalidUpdateUser> {
        if username.is_none() && email.is_none() {
            return Err(InvalidUpdateUser::NoUpdates);
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
pub struct DeleteUser(Uuid);

impl DeleteUser {
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
