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
//! 3. System tries the update; the database refuses a duplicate
//! 4. On success the email is updated; on a duplicate nothing changes and the
//!    address's owner is told, so the caller cannot learn that it is taken
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

use thiserror::Error;
use uuid::Uuid;
use zwipe_core::domain::auth::models::secret::Secret;
use zwipe_core::domain::user::User;
use zwipe_core::domain::{Email, InvalidEmail};

use crate::domain::auth::requests::authenticate_user::AuthenticateUserError;

/// Errors that can occur while constructing a [`ChangeEmail`] request.
#[derive(Debug, Error)]
pub enum InvalidChangeEmail {
    /// Invalid user ID format.
    #[error(transparent)]
    Id(#[from] uuid::Error),
    /// Invalid email format.
    #[error(transparent)]
    Email(#[from] InvalidEmail),
}

/// Errors that can occur during email change execution.
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
    /// Another user already has this email address. The service turns this
    /// into [`ChangeEmailOutcome::Taken`]; it never reaches the handler.
    #[error("user with that email already exists")]
    Duplicate,
}

/// What the service did with a change-email request.
///
/// Both arms carry the user as the caller sees them afterwards, so the handler
/// answers the same way for each and the response cannot say whether an
/// address is registered.
#[derive(Debug)]
pub enum ChangeEmailOutcome {
    /// The address was free. The user now carries it, unverified.
    Changed(User),
    /// The address belongs to another account. Nothing changed and that
    /// account's owner was told; the caller is returned as they were.
    Taken(User),
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
    pub email: Email,
    /// Current password for verification (not validated; verified at the service layer).
    pub password: Secret,
}

impl ChangeEmail {
    /// Creates a new email change request with validation.
    ///
    /// # Parameters
    ///
    /// - `user_id`: UUID of the user
    /// - `email`: New email address (will be validated)
    /// - `password`: Current password for verification (verified at the service layer)
    ///
    /// # Errors
    ///
    /// Returns [`InvalidChangeEmail`] if:
    /// - Email format is invalid
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
        let email = Email::new(email)?;
        let password = Secret::new(password);
        Ok(Self {
            user_id,
            email,
            password,
        })
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;

    #[test]
    fn test_change_email_new_succeeds_with_valid_inputs() {
        let user_id = Uuid::new_v4();
        let result = ChangeEmail::new(user_id, "newemail@example.com", "SecurePass123!");
        assert!(result.is_ok());
        let req = result.unwrap();
        assert_eq!(req.user_id, user_id);
        assert_eq!(req.email.to_string(), "newemail@example.com");
    }

    #[test]
    fn test_change_email_new_rejects_invalid_email() {
        let user_id = Uuid::new_v4();
        let result = ChangeEmail::new(user_id, "not-an-email", "SecurePass123!");
        assert!(matches!(result, Err(InvalidChangeEmail::Email(_))));
    }

    #[test]
    fn test_change_email_new_accepts_legacy_weak_password() {
        // The current password is verified at the service layer, not policy-gated here,
        // so a legacy password that predates the current policy must not be rejected.
        let user_id = Uuid::new_v4();
        let result = ChangeEmail::new(user_id, "newemail@example.com", "weak");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().password.read(), "weak");
    }
}
