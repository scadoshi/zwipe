//! Password change operation with current password verification.
//!
//! Allows users to update their password while requiring verification of the current password.
//! This prevents unauthorized password changes even if an attacker has session access.
//!
//! # Security Flow
//!
//! 1. User provides current password + new password
//! 2. System verifies current password matches stored hash
//! 3. New password is validated and hashed with fresh salt
//! 4. Password hash is updated in database
//! 5. All existing sessions remain valid (user is not logged out)
//!
//! # Important
//!
//! Current password is NOT validated against security policy. This allows users
//! with legacy weak passwords to change to stronger ones without being locked out.
//!
//! # Example
//!
//! ```rust,ignore
//! use zwipe::domain::auth::models::change_password::ChangePassword;
//!
//! let change = ChangePassword::new(
//!     user_id,
//!     "OldPassword123!",
//!     "NewPassword456!"
//! )?;
//! ```

use zwipe_core::domain::auth::password::InvalidPassword;
use thiserror::Error;

#[cfg(feature = "zerver")]
use crate::domain::auth::{
    models::password::HashedPassword,
    requests::authenticate_user::AuthenticateUserError,
};
#[cfg(feature = "zerver")]
use uuid::Uuid;

/// Errors that can occur while constructing a [`ChangePassword`] request.
#[derive(Debug, Error)]
pub enum InvalidChangePassword {
    /// New password doesn't meet security requirements.
    #[error(transparent)]
    Password(InvalidPassword),
    /// New password is the same as the current one.
    #[error("new password must be different from your current password")]
    SameAsCurrent,
    /// Password hashing failed (Argon2id error).
    #[error("failed to hash password: {0}")]
    FailedPasswordHash(anyhow::Error),
}

/// Errors that can occur during password change execution.
#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum ChangePasswordError {
    /// User ID doesn't exist in database.
    #[error("user not found")]
    UserNotFound,
    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),
    /// Current password verification failed.
    #[error(transparent)]
    AuthenticateUserError(AuthenticateUserError),
}

#[cfg(feature = "zerver")]
impl From<AuthenticateUserError> for ChangePasswordError {
    fn from(value: AuthenticateUserError) -> Self {
        Self::AuthenticateUserError(value)
    }
}

#[cfg(feature = "zerver")]
impl From<sqlx::Error> for ChangePasswordError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => Self::UserNotFound,
            e => Self::Database(e.into()),
        }
    }
}

/// Request to change a user's password.
///
/// Contains the user ID, current password (for verification), and
/// pre-hashed new password ready for database storage.
///
/// # Security Note
///
/// Current password is intentionally NOT validated against security policy.
/// This allows users with legacy weak passwords to upgrade to stronger ones
/// without being locked out of password changes.
#[cfg(feature = "zerver")]
#[derive(Debug)]
pub struct ChangePassword {
    /// The user whose password should be changed.
    pub user_id: Uuid,
    /// Current password (plaintext) for verification.
    pub current_password: String,
    /// New password already hashed with Argon2id + fresh salt.
    pub new_password_hash: HashedPassword,
}
#[cfg(feature = "zerver")]
impl ChangePassword {
    /// Creates a new password change request with validation and hashing.
    ///
    /// # Parameters
    ///
    /// - `user_id`: UUID of the user
    /// - `current_password`: Current password for verification (NOT validated)
    /// - `new_password`: New password (will be validated and hashed)
    ///
    /// # Errors
    ///
    /// Returns [`InvalidChangePassword`] if:
    /// - New password doesn't meet security requirements
    /// - Password hashing fails (unlikely - Argon2id error)
    ///
    /// # Security Notes
    ///
    /// - Current password is NOT validated to allow legacy password changes
    /// - New password IS fully validated (length, complexity, not common)
    /// - New password is hashed immediately with fresh salt
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let change = ChangePassword::new(
    ///     user_id,
    ///     "OldPassword123!",
    ///     "NewSecurePassword456!"
    /// )?;
    /// ```
    pub fn new(
        user_id: Uuid,
        current_password: impl AsRef<str>,
        new_password: impl AsRef<str>,
    ) -> Result<Self, InvalidChangePassword> {
        use crate::domain::auth::models::password::Password;

        // Same-password check happens on plaintext, before hashing. It only blocks a
        // genuine no-op: if the typed current password is wrong, service-layer
        // authentication rejects the request anyway.
        if new_password.as_ref() == current_password.as_ref() {
            return Err(InvalidChangePassword::SameAsCurrent);
        }

        let new_password = Password::new(new_password).map_err(InvalidChangePassword::Password)?;
        // No validation of current password - allows users with weak passwords to change
        // to stronger ones without being locked out
        let current_password = current_password.as_ref().to_string();
        let new_password_hash = HashedPassword::generate(new_password)
            .map_err(|e| InvalidChangePassword::FailedPasswordHash(e.into()))?;

        Ok(Self {
            user_id,
            current_password,
            new_password_hash,
        })
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "zerver")]
    use super::*;
    #[cfg(feature = "zerver")]
    use uuid::Uuid;

    #[cfg(feature = "zerver")]
    #[test]
    fn test_change_password_new_succeeds_with_valid_inputs() {
        let user_id = Uuid::new_v4();
        let result = ChangePassword::new(user_id, "OldPass!", "NewSecure123!");
        assert!(result.is_ok());
        let req = result.unwrap();
        assert_eq!(req.user_id, user_id);
        assert_eq!(req.current_password, "OldPass!");
    }

    #[cfg(feature = "zerver")]
    #[test]
    fn test_change_password_new_rejects_invalid_new_password() {
        let user_id = Uuid::new_v4();
        let result = ChangePassword::new(user_id, "OldPass!", "short");
        assert!(matches!(result, Err(InvalidChangePassword::Password(_))));
    }

    #[cfg(feature = "zerver")]
    #[test]
    fn test_change_password_new_accepts_any_current_password() {
        // Current password is NOT validated against security policy
        let user_id = Uuid::new_v4();
        let result = ChangePassword::new(user_id, "w", "NewSecure123!");
        assert!(result.is_ok());
    }

    #[cfg(feature = "zerver")]
    #[test]
    fn test_change_password_new_rejects_same_as_current() {
        let user_id = Uuid::new_v4();
        let result = ChangePassword::new(user_id, "NewSecure123!", "NewSecure123!");
        assert!(matches!(result, Err(InvalidChangePassword::SameAsCurrent)));
    }

    #[cfg(feature = "zerver")]
    #[test]
    fn test_change_password_new_stores_hashed_new_password() {
        let user_id = Uuid::new_v4();
        let req = ChangePassword::new(user_id, "OldPass!", "NewSecure123!").unwrap();
        // Hash must not contain plaintext
        assert!(!req.new_password_hash.to_string().contains("NewSecure123!"));
        // Hash should be Argon2 PHC format
        assert!(req.new_password_hash.to_string().starts_with("$argon2"));
    }
}
