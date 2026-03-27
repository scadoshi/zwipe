//! User authentication (login) operation.
//!
//! This module handles user login by verifying credentials and creating a session.
//! Authentication supports login via email or username.
//!
//! # Authentication Flow
//!
//! 1. Client provides identifier (email or username) and password
//! 2. Service finds user by identifier
//! 3. Service verifies password against stored Argon2 hash
//! 4. Service creates session with access and refresh tokens
//! 5. Client receives session for authenticated API access
//!
//! # Identifier Flexibility
//!
//! Users can authenticate with either:
//! - Email address: `alice@example.com`
//! - Username: `alice`
//!
//! The service automatically determines which type was provided.
//!
//! # Security
//!
//! - **Password Verification**: Constant-time comparison using Argon2
//! - **No User Enumeration**: Generic error messages prevent identifying valid usernames
//! - **Rate Limiting**: Should be implemented at HTTP layer (not in domain)
//!
//! # Example
//!
//! ```rust,ignore
//! use zwipe::domain::auth::models::authenticate_user::AuthenticateUser;
//!
//! // Login with email
//! let request = AuthenticateUser::new("alice@example.com", "SecurePass123!")?;
//! let session = auth_service.authenticate_user(request).await?;
//!
//! // Or login with username
//! let request = AuthenticateUser::new("alice", "SecurePass123!")?;
//! let session = auth_service.authenticate_user(request).await?;
//!
//! // Session contains tokens for API access
//! println!("Logged in as: {}", session.user.username);
//! ```

use crate::domain::auth::models::password::{InvalidPassword, Password};
#[cfg(feature = "zerver")]
use crate::domain::auth::models::{
    change_email::ChangeEmail, change_password::ChangePassword, change_username::ChangeUsername,
    delete_user::DeleteUser, session::create_session::CreateSessionError,
};
use thiserror::Error;

#[cfg(feature = "zerver")]
/// Errors that can occur during user authentication.
///
/// Authentication involves multiple steps (user lookup, password verification,
/// session creation), each of which can fail. These errors help distinguish
/// between user errors (wrong password) and system errors (database failure).
#[derive(Debug, Error)]
pub enum AuthenticateUserError {
    /// No user found with the provided identifier (email or username).
    ///
    /// For security, this uses the same error message as `InvalidPassword` to prevent
    /// user enumeration attacks (attackers can't determine which usernames exist).
    #[error("user not found")]
    UserNotFound,

    /// The provided password doesn't match the stored password hash.
    ///
    /// This is the expected failure case for incorrect login attempts.
    #[error("invalid password")]
    InvalidPassword,

    /// Database operation failed during authentication.
    #[error(transparent)]
    Database(anyhow::Error),

    /// User was found but database returned invalid/corrupted data.
    ///
    /// This indicates database schema issues or data corruption.
    #[error("user found but database returned invalid object: {0}")]
    UserFromDb(anyhow::Error),

    /// Password hash verification operation failed.
    ///
    /// This is distinct from `InvalidPassword` - it means the verification
    /// process itself failed, not that the password was wrong.
    #[error("failed to verify password: {0}")]
    FailedToVerify(anyhow::Error),

    /// Failed to generate JWT access token for the session.
    #[error("failed to generate access token: {0}")]
    FailedAccessToken(anyhow::Error),

    /// Failed to create session after successful password verification.
    #[error(transparent)]
    CreateSessionError(#[from] CreateSessionError),

    /// Account is temporarily locked after too many failed login attempts.
    #[error("account temporarily locked")]
    AccountLocked,
}

/// Validation errors when constructing an [`AuthenticateUser`] request.
///
/// These are client-side validation failures before attempting authentication.
#[derive(Debug, Error)]
pub enum InvalidAuthenticateUser {
    /// No identifier (email/username) was provided.
    ///
    /// The identifier field is required and cannot be empty.
    #[error("identifier must be present")]
    MissingIdentifier,

    /// The password doesn't meet validation requirements.
    ///
    /// Note: For login, password validation is more lenient than registration
    /// (only checks it's not empty), since existing users may have passwords
    /// created under older policies.
    #[error(transparent)]
    Password(InvalidPassword),
}

impl From<InvalidPassword> for InvalidAuthenticateUser {
    fn from(value: InvalidPassword) -> Self {
        Self::Password(value)
    }
}

/// Request to authenticate a user and create a session.
///
/// This type validates the password (ensuring it meets basic requirements) but
/// accepts any non-empty string as an identifier. The service layer determines
/// whether the identifier is an email or username.
///
/// # Identifier Handling
///
/// The identifier can be:
/// - **Email**: `alice@example.com` - service queries by email
/// - **Username**: `alice` - service queries by username
/// - **User ID**: `550e8400-...` - service queries by ID (used for re-authentication)
///
/// The service automatically detects which type and queries accordingly.
///
/// # Re-authentication
///
/// This type is also used for re-authentication during sensitive operations
/// (password change, email change, etc.). The `From` implementations convert
/// those requests into authentication requests for password verification.
///
/// # Example
///
/// ```rust,ignore
/// use zwipe::domain::auth::models::authenticate_user::AuthenticateUser;
///
/// // Login with email
/// let request = AuthenticateUser::new(
///     "alice@example.com",
///     "SecurePass123!"
/// )?;
///
/// // Login with username
/// let request = AuthenticateUser::new("alice", "SecurePass123!")?;
///
/// // Authenticate
/// let session = auth_service.authenticate_user(request).await?;
/// ```
#[derive(Debug)]
pub struct AuthenticateUser {
    /// Email, username, or user ID to identify the user.
    ///
    /// The service layer determines which type this is and queries accordingly.
    pub identifier: String,

    /// The user's plaintext password to verify.
    ///
    /// This is verified against the stored Argon2 hash. The plaintext is
    /// only held temporarily during verification and never stored.
    pub password: String,
}

impl AuthenticateUser {
    /// Creates a new authentication request.
    ///
    /// Validates that both identifier and password are provided and that the
    /// password meets basic requirements.
    ///
    /// # Arguments
    ///
    /// * `identifier` - Email, username, or user ID (cannot be empty)
    /// * `password` - User's password (validated for basic requirements)
    ///
    /// # Errors
    ///
    /// Returns [`InvalidAuthenticateUser`] if:
    /// - Identifier is empty
    /// - Password doesn't meet validation requirements
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let request = AuthenticateUser::new("alice", "SecurePass123!")?;
    /// ```
    pub fn new(
        identifier: impl AsRef<str>,
        password: impl AsRef<str>,
    ) -> Result<Self, InvalidAuthenticateUser> {
        let identifier = identifier.as_ref();
        if identifier.is_empty() {
            return Err(InvalidAuthenticateUser::MissingIdentifier);
        }
        let password = Password::new(password)?;
        Ok(AuthenticateUser {
            identifier: identifier.to_string(),
            password: password.read().to_string(),
        })
    }
}

// ==========================================
// Re-authentication conversions
// ==========================================
//
// Sensitive operations (changing password, email, username, or deleting account)
// require re-authentication with the current password. These conversions allow
// those requests to be verified using the authentication service.

#[cfg(feature = "zerver")]
impl From<&ChangePassword> for AuthenticateUser {
    /// Converts a password change request into an authentication request.
    ///
    /// Used to verify the user's current password before allowing the change.
    fn from(value: &ChangePassword) -> Self {
        Self {
            identifier: value.user_id.to_string(),
            password: value.current_password.to_owned(),
        }
    }
}

#[cfg(feature = "zerver")]
impl From<&ChangeUsername> for AuthenticateUser {
    /// Converts a username change request into an authentication request.
    ///
    /// Used to verify the user's password before allowing the username change.
    fn from(value: &ChangeUsername) -> Self {
        Self {
            identifier: value.user_id.to_string(),
            password: value.password.to_string(),
        }
    }
}

#[cfg(feature = "zerver")]
impl From<&ChangeEmail> for AuthenticateUser {
    /// Converts an email change request into an authentication request.
    ///
    /// Used to verify the user's password before allowing the email change.
    fn from(value: &ChangeEmail) -> Self {
        Self {
            identifier: value.user_id.to_string(),
            password: value.password.to_string(),
        }
    }
}

#[cfg(feature = "zerver")]
impl From<&DeleteUser> for AuthenticateUser {
    /// Converts an account deletion request into an authentication request.
    ///
    /// Used to verify the user's password before allowing irreversible account deletion.
    fn from(value: &DeleteUser) -> Self {
        Self {
            identifier: value.user_id.to_string(),
            password: value.password.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authenticate_user_new_succeeds_with_username() {
        let result = AuthenticateUser::new("alice", "SecurePass123!");
        assert!(result.is_ok());
        let req = result.unwrap();
        assert_eq!(req.identifier, "alice");
        assert_eq!(req.password, "SecurePass123!");
    }

    #[test]
    fn test_authenticate_user_new_succeeds_with_email() {
        let result = AuthenticateUser::new("alice@example.com", "SecurePass123!");
        assert!(result.is_ok());
    }

    #[test]
    fn test_authenticate_user_new_rejects_empty_identifier() {
        let result = AuthenticateUser::new("", "SecurePass123!");
        assert!(matches!(result, Err(InvalidAuthenticateUser::MissingIdentifier)));
    }

    #[test]
    fn test_authenticate_user_new_rejects_invalid_password() {
        let result = AuthenticateUser::new("alice", "short");
        assert!(matches!(result, Err(InvalidAuthenticateUser::Password(_))));
    }

    #[test]
    fn test_invalid_authenticate_user_from_invalid_password() {
        use crate::domain::auth::models::password::InvalidPassword;
        let err = InvalidAuthenticateUser::from(InvalidPassword::TooShort);
        assert!(matches!(err, InvalidAuthenticateUser::Password(_)));
    }

    #[cfg(feature = "zerver")]
    #[test]
    fn test_authenticate_user_from_change_password() {
        use crate::domain::auth::models::change_password::ChangePassword;
        use uuid::Uuid;
        let user_id = Uuid::new_v4();
        let req = ChangePassword::new(user_id, "OldPass!", "NewSecure123!").unwrap();
        let auth: AuthenticateUser = AuthenticateUser::from(&req);
        assert_eq!(auth.identifier, user_id.to_string());
        assert_eq!(auth.password, "OldPass!");
    }

    #[cfg(feature = "zerver")]
    #[test]
    fn test_authenticate_user_from_change_username() {
        use crate::domain::auth::models::change_username::ChangeUsername;
        use uuid::Uuid;
        let user_id = Uuid::new_v4();
        let req = ChangeUsername::new(user_id, "newname", "SecurePass123!").unwrap();
        let auth: AuthenticateUser = AuthenticateUser::from(&req);
        assert_eq!(auth.identifier, user_id.to_string());
        assert_eq!(auth.password, "SecurePass123!");
    }

    #[cfg(feature = "zerver")]
    #[test]
    fn test_authenticate_user_from_change_email() {
        use crate::domain::auth::models::change_email::ChangeEmail;
        use uuid::Uuid;
        let user_id = Uuid::new_v4();
        let req = ChangeEmail::new(user_id, "new@example.com", "SecurePass123!").unwrap();
        let auth: AuthenticateUser = AuthenticateUser::from(&req);
        assert_eq!(auth.identifier, user_id.to_string());
        assert_eq!(auth.password, "SecurePass123!");
    }

    #[cfg(feature = "zerver")]
    #[test]
    fn test_authenticate_user_from_delete_user() {
        use crate::domain::auth::models::delete_user::DeleteUser;
        use uuid::Uuid;
        let user_id = Uuid::new_v4();
        let req = DeleteUser::new(user_id, "SomePassword!").unwrap();
        let auth: AuthenticateUser = AuthenticateUser::from(&req);
        assert_eq!(auth.identifier, user_id.to_string());
        assert_eq!(auth.password, "SomePassword!");
    }
}
