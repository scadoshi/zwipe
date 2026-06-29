//! Username change operation with password confirmation.
//!
//! Allows users to update their username while requiring password verification
//! for security. This prevents unauthorized username changes even if an attacker
//! has session access but not the password.
//!
//! # Security Flow
//!
//! 1. User provides new username + current password
//! 2. Username is validated (length, profanity, format)
//! 3. System verifies password matches current hash
//! 4. System checks new username isn't already taken
//! 5. Username is updated in database
//!
//! # Example
//!
//! ```rust,ignore
//! use zwipe::domain::auth::models::change_username::ChangeUsername;
//!
//! let change = ChangeUsername::new(
//!     user_id,
//!     "newusername",
//!     "CurrentPassword123!"
//! )?;
//! ```

#[cfg(feature = "zerver")]
use crate::domain::auth::requests::authenticate_user::AuthenticateUserError;
use zwipe_core::domain::user::username::{InvalidUsername, Username};
use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur while constructing a [`ChangeUsername`] request.
#[derive(Debug, Error)]
pub enum InvalidChangeUsername {
    /// Invalid user ID format.
    #[error(transparent)]
    Id(#[from] uuid::Error),
    /// Username doesn't meet requirements (length, profanity, format).
    #[error(transparent)]
    Username(#[from] InvalidUsername),
}

/// Errors that can occur during username change execution.
#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum ChangeUsernameError {
    /// User ID doesn't exist in database.
    #[error("user not found")]
    UserNotFound,
    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),
    /// Another user already has this username.
    #[error("user with that username already exists")]
    Duplicate,
    /// Database returned invalid user data after update.
    #[error("database returned invalid object: {0}")]
    UserFromDb(anyhow::Error),
    /// Password verification failed or user not found during authentication.
    #[error(transparent)]
    AuthenticateUserError(#[from] AuthenticateUserError),
}

/// Request to change a user's username.
///
/// Requires password confirmation to prevent unauthorized changes.
///
/// # Security Note
///
/// Password verification happens at the service layer. This prevents
/// username changes even if an attacker has an active session but
/// doesn't know the password.
#[derive(Debug)]
pub struct ChangeUsername {
    /// The user whose username should be changed.
    pub user_id: Uuid,
    /// The new username (already validated).
    pub new_username: Username,
    /// Current password for verification (not validated; verified at the service layer).
    pub password: String,
}

impl ChangeUsername {
    /// Creates a new username change request with validation.
    ///
    /// # Parameters
    ///
    /// - `user_id`: UUID of the user
    /// - `new_username`: New username (will be validated)
    /// - `password`: Current password for verification (verified at the service layer)
    ///
    /// # Errors
    ///
    /// Returns [`InvalidChangeUsername`] if:
    /// - Username doesn't meet requirements (3-20 chars, no profanity)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let change = ChangeUsername::new(
    ///     user_id,
    ///     "newusername",
    ///     "CurrentPassword123!"
    /// )?;
    /// ```
    pub fn new(
        user_id: Uuid,
        new_username: impl AsRef<str>,
        password: impl AsRef<str>,
    ) -> Result<Self, InvalidChangeUsername> {
        let new_username = Username::new(new_username)?;
        let password = password.as_ref().to_string();
        Ok(Self {
            user_id,
            new_username,
            password,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_change_username_new_succeeds_with_valid_inputs() {
        let user_id = Uuid::new_v4();
        let result = ChangeUsername::new(user_id, "newusername", "SecurePass123!");
        assert!(result.is_ok());
        let req = result.unwrap();
        assert_eq!(req.user_id, user_id);
        assert_eq!(req.new_username.to_string(), "newusername");
    }

    #[test]
    fn test_change_username_new_rejects_too_short_username() {
        let user_id = Uuid::new_v4();
        let result = ChangeUsername::new(user_id, "ab", "SecurePass123!");
        assert!(matches!(result, Err(InvalidChangeUsername::Username(_))));
    }

    #[test]
    fn test_change_username_new_accepts_legacy_weak_password() {
        // The current password is verified at the service layer, not policy-gated here,
        // so a legacy password that predates the current policy must not be rejected.
        let user_id = Uuid::new_v4();
        let result = ChangeUsername::new(user_id, "newusername", "weak");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().password, "weak");
    }
}
