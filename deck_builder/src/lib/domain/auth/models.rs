pub mod jwt;
pub mod password;

use std::str::FromStr;

use email_address::EmailAddress;

use crate::domain::auth::models::password::{HashedPasswordError, Password, PasswordError};
use crate::domain::user::models::{
    AuthenticateUserRequestError, AuthenticateUserSuccessResponseError, User, UserId, UserName,
};
use crate::domain::{
    auth::models::{
        jwt::{Jwt, JwtError},
        password::HashedPassword,
    },
    user::models::UserNameEmptyError,
};

// =============================================================================
// ERROR TYPES
// =============================================================================

/// Actual errors encountered while registering a user
#[derive(Debug, Error)]
pub enum RegisterUserError {
    #[error("User with name or email already exists")]
    Duplicate,
    #[error("Database issues: {0}")]
    DatabaseIssues(anyhow::Error),
    #[error("User created but database returned invalid User. DatabaseUser -> User conversion error: {0}")]
    InvalidUserFromDatabase(anyhow::Error),
    #[error("Failed to generate JWT: {0}")]
    FailedJwt(anyhow::Error),
}

/// For constructor of RegisterUserRequest
#[derive(Debug, Error)]
pub enum RegisterUserRequestError {
    #[error(transparent)]
    InvalidUserName(UserNameEmptyError),
    #[error(transparent)]
    InvalidEmail(email_address::Error),
    #[error(transparent)]
    InvalidPassword(PasswordError),
    #[error(transparent)]
    FailedPasswordHash(password_hash::Error),
}

/// Actual errors you might encounter while doing user authentication
#[derive(Debug, Error)]
pub enum AuthenticateUserError {
    #[error("User not found")]
    UserNotFound,
    #[error("Invalid password")]
    InvalidPassword,
    #[error("Database issues: {0}")]
    DatabaseIssues(anyhow::Error),
    #[error("User found but database returned invalid User. DatabaseUserWithPasswordHash -> UserWithPasswordHash conversion error: {0}")]
    InvalidUserFromDatabase(anyhow::Error),
    #[error("Failed to verify password: {0}")]
    FailedToVerify(anyhow::Error),
    #[error("Failed to generate JWT: {0}")]
    FailedJwt(anyhow::Error),
}

/// For constructor of AuthenticateUserRequest
#[derive(Debug, Error)]
pub enum AuthenticateUserRequestError {
    #[error("Identifier must be present")]
    MissingIdentifier,
    #[error("Password must be present")]
    MissingPassword,
}

/// For constructor of AuthenticateUserSuccessResponse
#[derive(Debug, Error)]
pub enum AuthenticateUserSuccessResponseError {
    #[error(transparent)]
    JwtError(JwtError),
}

// =============================================================================
// REQUEST/RESPONSE TYPES
// =============================================================================

#[derive(Debug, Clone)]
pub struct RegisterUserRequest {
    pub username: UserName,
    pub email: EmailAddress,
    pub password_hash: HashedPassword,
}

impl RegisterUserRequest {
    pub fn new(
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<Self, RegisterUserRequestError> {
        let username =
            UserName::new(username).map_err(|e| RegisterUserRequestError::InvalidUserName(e))?;
        let email =
            EmailAddress::from_str(email).map_err(|e| RegisterUserRequestError::InvalidEmail(e))?;
        let password =
            Password::new(password).map_err(|e| RegisterUserRequestError::InvalidPassword(e))?;
        let password_hash = HashedPassword::generate(password)
            .map_err(|e| RegisterUserRequestError::FailedPasswordHash(e))?;

        Ok(RegisterUserRequest {
            username,
            email,
            password_hash,
        })
    }
}

/// Authentication request with identifier (email/username) and password
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct AuthenticateUserRequest {
    pub identifier: String,
    pub password: String,
}

impl AuthenticateUserRequest {
    pub fn new(identifier: &str, password: &str) -> Result<Self, AuthenticateUserRequestError> {
        if identifier.is_empty() {
            return Err(AuthenticateUserRequestError::MissingIdentifier);
        }
        if password.is_empty() {
            return Err(AuthenticateUserRequestError::MissingPassword);
        }
        Ok(AuthenticateUserRequest {
            identifier: identifier.to_string(),
            password: password.to_string(),
        })
    }
}

/// Successful authentication response containing user data and JWT token
/// Registering also uses this
#[derive(Debug, Clone)]
pub struct AuthenticateUserSuccessResponse {
    pub user: User,
    pub token: Jwt,
    pub expires_at: usize,
}

impl AuthenticateUserSuccessResponse {
    pub fn new(
        user: User,
        token_string: String,
        expires_at: usize,
    ) -> Result<Self, AuthenticateUserSuccessResponseError> {
        let token = Jwt::new(&token_string)
            .map_err(|e| AuthenticateUserSuccessResponseError::JwtError(e))?;
        Ok(AuthenticateUserSuccessResponse {
            user,
            token,
            expires_at,
        })
    }
}

// =============================================================================
// MAIN DOMAIN ENTITIES
// =============================================================================

/// User entity with password hash for authentication operations
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
