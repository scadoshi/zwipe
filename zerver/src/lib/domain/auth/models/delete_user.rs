//! User account deletion with password confirmation.
//!
//! Allows users to permanently delete their account while requiring password verification.
//! This is an irreversible operation that removes all user data.
//!
//! # Security Flow
//!
//! 1. User provides password for confirmation
//! 2. System verifies password matches current hash
//! 3. All user data is deleted (cascading to decks, sessions, etc.)
//! 4. User is logged out from all sessions
//!
//! # Important
//!
//! This is a destructive operation. Consider implementing:
//! - Soft deletion (marking as deleted, data retention)
//! - Grace period (delayed deletion, allows undo)
//! - Data export before deletion (GDPR requirement)
//!
//! # Example
//!
//! ```rust,ignore
//! use zwipe::domain::auth::models::delete_user::DeleteUser;
//!
//! let delete = DeleteUser::new(
//!     user_id,
//!     "MyPassword123!"
//! )?;
//! ```

#[cfg(feature = "zerver")]
use crate::domain::auth::models::authenticate_user::AuthenticateUserError;
use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur during user deletion execution.
#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum DeleteUserError {
    /// User ID doesn't exist in database.
    #[error("user not found")]
    NotFound,
    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),
    /// Password verification failed.
    #[error(transparent)]
    AuthenticateUserError(#[from] AuthenticateUserError),
}

/// Errors that can occur while constructing a [`DeleteUser`] request.
#[derive(Debug, Error)]
pub enum InvalidDeleteUser {
    /// Invalid user ID format.
    #[error(transparent)]
    Userid(uuid::Error),
    /// Invalid password (empty or whitespace-only).
    #[error("invalid password")]
    Password,
}

impl From<uuid::Error> for InvalidDeleteUser {
    fn from(value: uuid::Error) -> Self {
        Self::Userid(value)
    }
}

/// Request to delete a user account.
///
/// Requires password confirmation to prevent accidental or unauthorized deletion.
///
/// # Security Note
///
/// Password is stored as plaintext string (not validated) because this is
/// a destructive operation - we want to verify the user knows their password
/// but don't want to prevent deletion due to validation issues.
#[derive(Debug, Clone)]
pub struct DeleteUser {
    /// The user to delete.
    pub user_id: Uuid,
    /// Password for verification (trimmed, not validated).
    pub password: String,
}

impl DeleteUser {
    /// Creates a new user deletion request.
    ///
    /// # Parameters
    ///
    /// - `user_id`: UUID of the user to delete
    /// - `password`: Password for verification (will be trimmed)
    ///
    /// # Errors
    ///
    /// Returns [`InvalidDeleteUser`] if:
    /// - User ID format is invalid
    ///
    /// Note: Password is intentionally NOT validated to allow deletion
    /// even with legacy weak passwords.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let delete = DeleteUser::new(
    ///     user_id,
    ///     "MyPassword123!"
    /// )?;
    /// ```
    pub fn new(user_id: Uuid, password: &str) -> Result<Self, InvalidDeleteUser> {
        let password = password.trim();
        Ok(Self {
            user_id,
            password: password.to_string(),
        })
    }
}
