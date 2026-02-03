//! Email change operation with password confirmation.
//!
//! Allows users to update their email address while requiring password verification
//! for security. This prevents unauthorized email changes even if an attacker has
//! session access but not the password.
//!
//! # Security Flow
//!
//! 1. User provides new email + current password
//! 2. System verifies password matches current hash
//! 3. System checks new email isn't already in use
//! 4. Email is updated in database
//!
//! # Example
//!
//! ```rust,ignore
//! use zwipe::domain::auth::models::change_email::ChangeEmail;
//!
//! let change = ChangeEmail::new(
//!     user_id,
//!     "newemail@example.com",
//!     "CurrentPassword123!"
//! )?;
//! ```

use email_address::EmailAddress;
use std::str::FromStr;
use thiserror::Error;
use uuid::Uuid;

#[cfg(feature = "zerver")]
use crate::domain::auth::models::authenticate_user::AuthenticateUserError;
use crate::domain::auth::models::password::{InvalidPassword, Password};

/// Errors that can occur while constructing a [`ChangeEmail`] request.
#[derive(Debug, Error)]
pub enum InvalidChangeEmail {
    /// Invalid user ID format.
    #[error(transparent)]
    Id(#[from] uuid::Error),
    /// Invalid email format.
    #[error(transparent)]
    Email(#[from] email_address::Error),
    /// Password doesn't meet security requirements.
    #[error(transparent)]
    Password(#[from] InvalidPassword),
}

/// Errors that can occur during email change execution.
#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum ChangeEmailError {
    /// User ID doesn't exist in database.
    #[error("user not found")]
    UserNotFound,
    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),
    /// Database returned invalid user data after update.
    #[error("user updated but database returned invalid object: {0}")]
    UserFromDb(anyhow::Error),
    /// Password verification failed or user not found during authentication.
    #[error(transparent)]
    AuthenticateUserError(#[from] AuthenticateUserError),
    /// Another user already has this email address.
    #[error("user with that email already exists")]
    Duplicate,
}

/// Request to change a user's email address.
///
/// Requires password confirmation to prevent unauthorized changes.
///
/// # Security Note
///
/// Password verification happens at the service layer using
/// [`AuthenticateUserError`]. This prevents email changes even
/// if an attacker has an active session but doesn't know the password.
#[derive(Debug)]
pub struct ChangeEmail {
    /// The user whose email should be changed.
    pub user_id: Uuid,
    /// The new email address (already validated).
    pub email: EmailAddress,
    /// Current password for verification (already validated).
    pub password: Password,
}

impl ChangeEmail {
    /// Creates a new email change request with validation.
    ///
    /// # Parameters
    ///
    /// - `user_id`: UUID of the user
    /// - `email`: New email address (will be validated)
    /// - `password`: Current password for verification (will be validated)
    ///
    /// # Errors
    ///
    /// Returns [`InvalidChangeEmail`] if:
    /// - Email format is invalid
    /// - Password doesn't meet security requirements (length, complexity, etc.)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let change = ChangeEmail::new(
    ///     user_id,
    ///     "newemail@example.com",
    ///     "CurrentPassword123!"
    /// )?;
    /// ```
    pub fn new(user_id: Uuid, email: &str, password: &str) -> Result<Self, InvalidChangeEmail> {
        let email = EmailAddress::from_str(email)?;
        let password = Password::new(password)?;
        Ok(Self {
            user_id,
            email,
            password,
        })
    }
}
