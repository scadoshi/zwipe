//! Authentication domain entities and value objects.
//!
//! This module contains entities and value objects used in authentication workflows.
//! Request types live in the sibling [`super::requests`] module.
//!
//! # Module Organization
//!
//! - [`access_token`]: JWT access token generation and validation
//! - [`password`]: Password validation, hashing, and verification
//! - [`refresh_token`]: Long-lived refresh token management
//! - [`session`]: Session entity and constants

pub mod access_token;
pub mod password;
/// Long-lived refresh token management (re-exported from zwipe-core).
pub mod refresh_token;
/// Session entity and constants (re-exported from zwipe-core).
pub mod session;

#[cfg(feature = "zerver")]
use crate::domain::auth::models::password::HashedPassword;
#[cfg(feature = "zerver")]
use crate::domain::user::models::{username::Username, User};
#[cfg(feature = "zerver")]
use chrono::NaiveDateTime;
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
    /// If set and in the future, the account is locked until this timestamp.
    pub lockout_until: Option<NaiveDateTime>,
    /// When the user's email was verified. `None` means not yet verified.
    pub email_verified_at: Option<NaiveDateTime>,
}

#[cfg(feature = "zerver")]
impl From<UserWithPasswordHash> for User {
    fn from(value: UserWithPasswordHash) -> Self {
        Self {
            id: value.id,
            username: value.username,
            email: value.email,
            email_verified_at: value.email_verified_at,
        }
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_user_from_user_with_password_hash_drops_password() {
        use crate::domain::auth::models::password::{HashedPassword, Password};
        use crate::domain::user::models::username::Username;
        use email_address::EmailAddress;
        use std::str::FromStr;
        use uuid::Uuid;

        let id = Uuid::new_v4();
        let username = Username::new("alice").unwrap();
        let email = EmailAddress::from_str("alice@example.com").unwrap();
        let password = Password::new("SecurePass123!").unwrap();
        let password_hash = HashedPassword::generate(password).unwrap();

        let with_hash = UserWithPasswordHash {
            id,
            username: username.clone(),
            email: email.clone(),
            password_hash,
            lockout_until: None,
            email_verified_at: None,
        };

        let user: User = with_hash.into();
        assert_eq!(user.id, id);
        assert_eq!(user.username, username);
        assert_eq!(user.email, email);
    }
}
