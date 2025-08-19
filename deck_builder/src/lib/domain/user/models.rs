use email_address::EmailAddress;
use std::{fmt::Display, str::FromStr};
use thiserror::Error;
use uuid::Uuid;

// =============================================================================
// ERRORS
// =============================================================================

#[derive(Debug, Error)]
pub enum UserNameError {
    #[error("Username must be present")]
    MissingUserName,
}

/// For constructor of CreateUserRequest
#[derive(Debug, Error)]
pub enum CreateUserRequestError {
    #[error(transparent)]
    InvalidUsername(UserNameError),
    #[error(transparent)]
    InvalidEmail(email_address::Error),
}

/// Actual errors encountered while creating a user
#[derive(Debug, Error)]
pub enum CreateUserError {
    #[error("User with name or email already exists")]
    Duplicate,
    #[error("Database issues: {0}")]
    DatabaseIssues(anyhow::Error),
    #[error("User created but database returned invalid User object. DatabaseUser -> User conversion error: {0}")]
    InvalidUserFromDatabase(anyhow::Error),
    #[error(transparent)]
    InvalidRequest(CreateUserRequestError),
}

/// Actual errors encountered while getting a user
#[derive(Debug, Error)]
pub enum GetUserError {
    #[error("User not found")]
    NotFound,
    #[error("Database issues: {0}")]
    DatabaseIssues(anyhow::Error),
    #[error("User found but database returned invalid User object. DatabaseUser -> User conversion error: {0}")]
    InvalidUserFromDatabase(anyhow::Error),
}

/// For constructor of UpdateUserRequest
#[derive(Debug, Error)]
pub enum UpdateUserRequestError {
    #[error(transparent)]
    InvalidId(uuid::Error),
    #[error(transparent)]
    InvalidUsername(UserNameError),
    #[error(transparent)]
    InvalidEmail(email_address::Error),
}

/// Actual errors encountered while updating a user
#[derive(Debug, Error)]
pub enum UpdateUserError {
    #[error("User with name or email already exists")]
    Duplicate,
    #[error("Database issues: {0}")]
    DatabaseIssues(anyhow::Error),
    #[error("User created but database returned invalid User. DatabaseUser -> User conversion error: {0}")]
    InvalidUserFromDatabase(anyhow::Error),
    #[error("User not found")]
    UserNotFound,
}

/// Actual errors encountered while deleting a user
#[derive(Debug, Error)]
pub enum DeleteUserRequestError {
    #[error("Id must be present")]
    MissingId,
    #[error("Failed to parse Uuid: {0}")]
    FailedUuid(uuid::Error),
}

#[derive(Debug, Error)]
pub enum DeleteUserError {
    #[error("User not found")]
    NotFound,
    #[error("Database issues: {0}")]
    DatabaseIssues(anyhow::Error),
}

// =============================================================================
// NEWTYPES
// =============================================================================

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
    pub username: UserName,
    pub email: EmailAddress,
}

impl CreateUserRequest {
    pub fn new(username: &str, email: &str) -> Result<Self, CreateUserRequestError> {
        let username =
            UserName::new(username).map_err(|e| CreateUserRequestError::InvalidUsername(e))?;
        let email =
            EmailAddress::from_str(email).map_err(|e| CreateUserRequestError::InvalidEmail(e))?;
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
    pub id: Uuid,
    pub username: Option<UserName>,
    pub email: Option<EmailAddress>,
}

impl UpdateUserRequest {
    fn new(
        id: &str,
        username_opt: Option<&str>,
        email_opt: Option<&str>,
    ) -> Result<Self, UpdateUserRequestError> {
        let id = Uuid::try_parse(id).map_err(|e| UpdateUserRequestError::InvalidId(e))?;
        let username = username_opt
            .map(|username_str| {
                UserName::new(username_str).map_err(|e| UpdateUserRequestError::InvalidUsername(e))
            })
            .transpose()?;

        let email = email_opt
            .map(|email_str| {
                EmailAddress::from_str(email_str)
                    .map_err(|e| UpdateUserRequestError::InvalidEmail(e))
            })
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
    fn new(id: &str) -> Result<Self, DeleteUserRequestError> {
        let trimmed = id.trim();
        if trimmed.is_empty() {
            return Err(DeleteUserRequestError::MissingId);
        }

        let id = Uuid::try_parse(trimmed).map_err(|e| DeleteUserRequestError::FailedUuid(e))?;
        Ok(Self(id))
    }

    pub fn id(&self) -> Uuid {
        self.0
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
