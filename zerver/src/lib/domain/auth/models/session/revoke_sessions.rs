//! Session revocation operation (logout from all devices).
//!
//! Deletes all sessions for a user, effectively logging them out from all devices.
//! This is the backend implementation of "logout everywhere" or "sign out all devices".
//!
//! # Use Cases
//!
//! - **User-Initiated**: User clicks "logout all devices" in settings
//! - **Security**: User suspects account compromise
//! - **Password Change**: Optional auto-logout after password change
//! - **Admin Action**: Support team revokes sessions for security
//!
//! # Security Note
//!
//! After revocation:
//! - All access tokens become useless (JWTs still valid but session check fails)
//! - All refresh tokens are deleted (cannot obtain new access tokens)
//! - User must re-authenticate to create new sessions
//!
//! # Example
//!
//! ```rust,ignore
//! use zwipe::domain::auth::models::session::revoke_sessions::RevokeSessions;
//!
//! // Revoke all sessions for a user
//! let revoke = RevokeSessions::new(user_id);
//! session_service.revoke_sessions(revoke).await?;
//! ```

use std::str::FromStr;
use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur while constructing a [`RevokeSessions`] request.
#[derive(Debug, Error)]
pub enum InvalidRevokeSessions {
    /// Invalid user ID format.
    #[error(transparent)]
    UserId(uuid::Error),
}

impl From<uuid::Error> for InvalidRevokeSessions {
    fn from(value: uuid::Error) -> Self {
        Self::UserId(value)
    }
}

/// Errors that can occur during session revocation execution.
#[derive(Debug, Error)]
pub enum RevokeSessionsError {
    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),
}

/// Request to revoke all sessions for a user (logout everywhere).
///
/// This is a simple wrapper around a user ID that represents the intent
/// to delete all active sessions for that user.
///
/// # Effect
///
/// - All sessions for this user are deleted from the database
/// - All refresh tokens are deleted (cannot get new access tokens)
/// - Access tokens become useless (session validation will fail)
/// - User must re-authenticate to create new sessions
#[derive(Debug, Clone)]
pub struct RevokeSessions {
    /// The user whose sessions should be revoked.
    pub user_id: Uuid,
}

impl RevokeSessions {
    /// Creates a new session revocation request.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let revoke = RevokeSessions::new(user_id);
    /// ```
    pub fn new(user_id: Uuid) -> Self {
        Self { user_id }
    }
}

impl FromStr for RevokeSessions {
    type Err = InvalidRevokeSessions;

    /// Parses a user ID string into a revocation request.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidRevokeSessions::UserId`] if the string is not a valid UUID.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let user_id = Uuid::try_parse(s)?;
        Ok(Self { user_id })
    }
}

impl From<Uuid> for RevokeSessions {
    fn from(value: Uuid) -> Self {
        Self::new(value)
    }
}
