//! Authentication domain models and value objects.
//!
//! This module contains all request/response types, entities, and value objects
//! used in authentication workflows.
//!
//! # Module Organization
//!
//! - [`access_token`]: JWT access token generation and validation
//! - [`authenticate_user`]: User login requests and responses
//! - [`change_email`]: Email change requests with password verification
//! - [`change_password`]: Password change requests with current password verification
//! - [`change_username`]: Username change requests with password verification
//! - [`delete_user`]: Account deletion requests with password verification
//! - [`password`]: Password validation, hashing, and verification
//! - [`refresh_token`]: Long-lived refresh token management
//! - [`register_user`]: New user registration requests
//! - [`session`]: Session creation, refresh, and management

pub mod access_token;
pub mod authenticate_user;
pub mod change_email;
pub mod change_password;
pub mod change_username;
pub mod delete_user;
pub mod password;
pub mod refresh_token;
pub mod register_user;
pub mod session;

#[cfg(feature = "zerver")]
use crate::domain::auth::models::password::HashedPassword;
#[cfg(feature = "zerver")]
use crate::domain::user::models::{username::Username, User};
#[cfg(feature = "zerver")]
use email_address::EmailAddress;
#[cfg(feature = "zerver")]
use uuid::Uuid;

#[cfg(feature = "zerver")]
/// User entity with password hash for authentication operations.
///
/// This internal domain type extends the public [`User`] entity with the password hash,
/// enabling password verification during authentication and account changes.
///
/// # Security
///
/// The password hash is an Argon2id hash and should never be exposed in API responses.
/// This type is only used internally within the auth service and repository layers.
///
/// # Conversion
///
/// Can be converted to a public [`User`] (without password hash) via [`From`].
///
/// # Fields
///
/// * `id` - Unique user identifier
/// * `username` - Validated username (3-20 chars, no profanity)
/// * `email` - Validated email address
/// * `password_hash` - Argon2id password hash with embedded salt
///
/// # Example
///
/// ```rust,ignore
/// // Retrieved from database during authentication
/// let user_with_hash: UserWithPasswordHash = user_repo.get_by_email(&email).await?;
///
/// // Verify password
/// user_with_hash.password_hash.verify(&provided_password)?;
///
/// // Convert to public user for response
/// let public_user: User = user_with_hash.into();
/// ```
#[derive(Debug)]
pub struct UserWithPasswordHash {
    /// Unique user identifier.
    pub id: Uuid,
    /// Validated username.
    pub username: Username,
    /// Validated email address.
    pub email: EmailAddress,
    /// Argon2id hashed password (never exposed in public APIs).
    pub password_hash: HashedPassword,
}

#[cfg(feature = "zerver")]
impl From<UserWithPasswordHash> for User {
    fn from(value: UserWithPasswordHash) -> Self {
        Self {
            id: value.id,
            username: value.username,
            email: value.email,
        }
    }
}
