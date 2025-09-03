pub mod jwt;
pub mod password;
use crate::domain::auth::models::password::{Password, PasswordError};
use crate::domain::auth::models::{
    jwt::{Jwt, JwtError},
    password::HashedPassword,
};
use crate::domain::user::models::{User, UserName, UserNameError};
use email_address::EmailAddress;
use serde::Serialize;
use std::str::FromStr;
use thiserror::Error;
use uuid::Uuid;

// ========
//  errors
// ========

/// errors encountered while registering a user
#[derive(Debug, Error)]
pub enum RegisterUserError {
    #[error("user with name or email already exists")]
    Duplicate,
    #[error(transparent)]
    Database(anyhow::Error),
    #[error("user created but database returned invalid object: {0}")]
    InvalidUserFromDatabase(anyhow::Error),
    #[error("failed to generate `JWT`: {0}")]
    FailedJwt(anyhow::Error),
    #[error(transparent)]
    InvalidRequest(RegisterUserRequestError),
}

/// errors encountered while constructing `RegisterUserRequest`
#[derive(Debug, Error)]
pub enum RegisterUserRequestError {
    #[error(transparent)]
    InvalidUsername(UserNameError),
    #[error(transparent)]
    InvalidEmail(email_address::Error),
    #[error(transparent)]
    InvalidPassword(PasswordError),
    #[error(transparent)]
    FailedPasswordHash(argon2::password_hash::Error),
}

/// errors encountered while authenticating a user
#[derive(Debug, Error)]
pub enum AuthenticateUserError {
    #[error("user not found")]
    UserNotFound,
    #[error("invalid password")]
    InvalidPassword,
    #[error(transparent)]
    Database(anyhow::Error),
    #[error("user found but database returned invalid object: {0}")]
    InvalidUserFromDatabase(anyhow::Error),
    #[error("failed to verify password: {0}")]
    FailedToVerify(anyhow::Error),
    #[error("failed to generate `JWT`: {0}")]
    FailedJwt(anyhow::Error),
}

/// errors encountered while constructing `AuthenticateUserRequest`
#[derive(Debug, Error)]
pub enum AuthenticateUserRequestError {
    #[error("identifier must be present")]
    MissingIdentifier,
    #[error("password must be present")]
    MissingPassword,
}

/// errors encountered while constructing `AuthenticateUserSuccessResponseError`
#[derive(Debug, Error)]
pub enum AuthenticateUserSuccessResponseError {
    #[error(transparent)]
    JwtError(JwtError),
}

/// errors encountered while constructing `ChangePasswordRequestError`
#[derive(Debug, Error)]
pub enum ChangePasswordRequestError {
    #[error(transparent)]
    InvalidId(uuid::Error),
    #[error(transparent)]
    InvalidPassword(PasswordError),
    #[error(transparent)]
    FailedPasswordHash(argon2::password_hash::Error),
}

/// errors encountered while changing password
#[derive(Debug, Error)]
pub enum ChangePasswordError {
    #[error("user not found")]
    UserNotFound,
    #[error(transparent)]
    Database(anyhow::Error),
}

// ========================
//  request/response types
// ========================

#[derive(Debug)]
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
            UserName::new(username).map_err(|e| RegisterUserRequestError::InvalidUsername(e))?;
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

/// authentication request with identifier (email/username) and password
#[derive(Debug)]
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

/// successful authentication response containing user data and JWT token
///
/// authentication and register user requeast use this
#[derive(Debug, Serialize, PartialEq)]
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

/// change password request
/// with idenifier and new password hash
#[derive(Debug)]
pub struct ChangePasswordRequest {
    pub id: Uuid,
    pub password_hash: HashedPassword,
}

impl ChangePasswordRequest {
    pub fn new(id: &str, new_password: &str) -> Result<Self, ChangePasswordRequestError> {
        let id = Uuid::try_parse(id).map_err(|e| ChangePasswordRequestError::InvalidId(e))?;
        let password = Password::new(new_password)
            .map_err(|e| ChangePasswordRequestError::InvalidPassword(e))?;
        let password_hash = HashedPassword::generate(password)
            .map_err(|e| ChangePasswordRequestError::FailedPasswordHash(e))?;

        Ok(Self { id, password_hash })
    }
}

// ======
//  main
// ======

/// user entity with password hash
/// for authentication operations
#[derive(Debug)]
pub struct UserWithPasswordHash {
    pub id: Uuid,
    pub username: UserName,
    pub email: EmailAddress,
    pub password_hash: Option<HashedPassword>,
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
