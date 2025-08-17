// =============================================================================
// IMPORTS
// =============================================================================

use anyhow::Context;
use email_address::{EmailAddress, Options};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use thiserror::Error;

use crate::domain::auth::models::{
    jwt::{Jwt, JwtError},
    password::{HashedPassword, Password},
};

// =============================================================================
// ERROR TYPES
// =============================================================================

/// Error when a username is empty or contains only whitespace
#[derive(Clone, Debug, Error)]
#[error("Username cannot be empty")]
pub struct UserNameEmptyError;

/// Error when a user ID is outside the valid range (0-999,999)
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("User ID must be between 0 and 999,999")]
pub struct UserIdError;

/// Errors that can occur during user creation
#[derive(Debug, Error)]
pub enum UserRegistrationError {
    #[error("User with name or email already exists")]
    Duplicate,
    #[error("Database issues: {0}")]
    DatabaseIssues(anyhow::Error),
    #[error("User created but then database returned an invalid User. DatabaseUser -> User conversion error: {0}")]
    InvalidUserFromDatabase(anyhow::Error),
    #[error("Failed to generate json web token: {0}")]
    FailedJwt(anyhow::Error),
}

/// Errors that can occur during user authentication
#[derive(Debug, Error)]
pub enum UserAuthenticationError {
    #[error("User not found")]
    UserNotFound,
    #[error("Invalid password")]
    InvalidPassword,
    #[error("Database issues: {0}")]
    DatabaseIssues(anyhow::Error),
    #[error("User found but then database returned an invalid User. DatabaseUserWithPasswordHash -> UserWithPasswordHash conversion error: {0} ")]
    InvalidUserFromDatabase(anyhow::Error),
    #[error("Failed to verify password: {0}")]
    FailedToVerify(anyhow::Error),
    #[error("Failed to generate json web token: {0}")]
    FailedJwt(anyhow::Error),
}

/// Errors that can occur when creating authentication requests
#[derive(Debug, Error)]
pub enum UserAuthenticationRequestError {
    #[error("Identifier must be present")]
    MissingIdentifier,
    #[error("Password must be present")]
    MissingPassword,
}

/// Errors that can occur when creating
#[derive(Debug, Error)]
pub enum UserAuthenticationSuccessResponseError {
    #[error(transparent)]
    JwtError(JwtError),
}

// =============================================================================
// DOMAIN NEWTYPES
// =============================================================================

/// A validated username that cannot be empty or contain only whitespace
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UserName(String);

impl UserName {
    /// Creates a new UserName after validation
    ///
    /// # Errors
    /// Returns `UserNameEmptyError` if the input is empty or only whitespace
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

/// A validated user ID within the range 0-999,999
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Copy)]
pub struct UserId(i32);

impl UserId {
    /// Creates a new UserId after validation
    ///
    /// # Errors
    /// Returns `UserIdError` if the ID is outside the valid range
    pub fn new(raw: i32) -> Result<Self, UserIdError> {
        if raw < 0 || raw > 999_999 {
            return Err(UserIdError);
        }
        Ok(Self(raw))
    }
}

/// Ensures validation logic is used during deserialization
impl<'de> Deserialize<'de> for UserId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = i32::deserialize(deserializer)?;
        UserId::new(raw).map_err(serde::de::Error::custom)
    }
}

/// Serializes only the inner value for external APIs
impl Serialize for UserId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

// =============================================================================
// REQUEST/RESPONSE TYPES
// =============================================================================

/// Request to create a new user with validated fields and hashed password
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct UserCreationRequest {
    pub email: EmailAddress,
    pub username: UserName,
    pub password_hash: HashedPassword,
}

impl UserCreationRequest {
    /// Creates a new user creation request with full validation and password hashing
    ///
    /// # Errors
    /// Returns error if username, email, or password validation fails
    pub fn new(username: &str, email: &str, password: &str) -> anyhow::Result<Self> {
        let username = UserName::new(username).context("Invalid username")?;
        let email = EmailAddress::parse_with_options(email, Options::default())
            .context("Invalid email address")?;
        let password_hash = Password::new(password)
            .context("Failed to create Password")?
            .hash()
            .context("Failed to create HashedPassword")?;
        Ok(UserCreationRequest {
            email,
            username,
            password_hash,
        })
    }
}

/// Request to authenticate a user with identifier (email/username) and password
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct UserAuthenticationRequest {
    pub identifier: String,
    pub password: String,
}

impl UserAuthenticationRequest {
    /// Creates a new authentication request with basic validation
    ///
    /// # Errors
    /// Returns error if identifier or password is empty
    pub fn new(identifier: &str, password: &str) -> Result<Self, UserAuthenticationRequestError> {
        if identifier.is_empty() {
            return Err(UserAuthenticationRequestError::MissingIdentifier);
        }
        if password.is_empty() {
            return Err(UserAuthenticationRequestError::MissingPassword);
        }
        Ok(UserAuthenticationRequest {
            identifier: identifier.to_string(),
            password: password.to_string(),
        })
    }
}

/// For authentication only
#[derive(Debug, Clone)]
pub struct UserWithPasswordHash {
    pub id: UserId,
    pub username: UserName,
    pub email: EmailAddress,
    pub password_hash: HashedPassword,
}

impl From<UserWithPasswordHash> for User {
    fn from(value: UserWithPasswordHash) -> Self {
        Self {
            id: value.id,
            username: value.username,
            email: value.email,
        }
    }
}

/// Successful authentication response containing user data and JWT token
#[derive(Debug, Clone)]
pub struct UserAuthenticationSuccessResponse {
    pub user: User,
    pub token: Jwt,
    pub expires_at: usize,
}

impl UserAuthenticationSuccessResponse {
    /// Creates a new authentication success response
    ///
    /// # Errors
    /// Returns error if token is not a valid Jwt
    pub fn new(
        user: User,
        token_string: String,
        expires_at: usize,
    ) -> Result<Self, UserAuthenticationSuccessResponseError> {
        let token = Jwt::new(&token_string)
            .map_err(|e| UserAuthenticationSuccessResponseError::JwtError(e))?;
        Ok(UserAuthenticationSuccessResponse {
            user,
            token,
            expires_at,
        })
    }
}

// =============================================================================
// MAIN DOMAIN ENTITIES
// =============================================================================

/// Core user entity representing an authenticated user in the system
#[derive(Debug, Clone)]
pub struct User {
    pub id: UserId,
    pub username: UserName,
    pub email: EmailAddress,
}
