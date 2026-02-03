//! Create session operation for authenticated users.
//!
//! This module provides the request type and errors for creating new sessions.
//! Sessions are created after successful authentication (login/registration) or
//! by administrators for user impersonation.
//!
//! # Session Creation Flow
//!
//! 1. Validate user exists and is active
//! 2. Enforce session maximum (delete oldest if > 5 sessions)
//! 3. Generate JWT access token with user claims
//! 4. Create refresh token with 14-day expiry
//! 5. Store refresh token hash in database
//! 6. Return complete session to client
//!
//! # Example
//!
//! ```rust,ignore
//! use zwipe::domain::auth::models::session::create_session::CreateSession;
//!
//! // Create session for user after authentication
//! let request = CreateSession::from(user_id);
//! let session = session_service.create_session(request).await?;
//!
//! // Session contains access token, refresh token, and user data
//! println!("New session for user: {}", session.user.username);
//! ```

#[cfg(feature = "zerver")]
use std::str::FromStr;

#[cfg(feature = "zerver")]
use crate::domain::{
    auth::models::{
        access_token::InvalidJwt, session::enforce_session_maximum::EnforceSessionMaximumError,
    },
    user::models::get_user::GetUserError,
};
#[cfg(feature = "zerver")]
use thiserror::Error;
#[cfg(feature = "zerver")]
use uuid::Uuid;

#[cfg(feature = "zerver")]
/// Validation errors when constructing a [`CreateSession`] request.
///
/// Currently only validates that the user ID is a valid UUID.
#[derive(Debug, Error)]
pub enum InvalidCreateSession {
    /// The provided user ID string is not a valid UUID.
    #[error(transparent)]
    UserId(#[from] uuid::Error),
}

#[cfg(feature = "zerver")]
/// Errors that can occur during session creation.
///
/// Session creation involves multiple operations: fetching the user, enforcing
/// session limits, generating tokens, and storing the refresh token. Each can fail.
#[derive(Debug, Error)]
pub enum CreateSessionError {
    /// Database operation failed (token storage, user fetch).
    #[error(transparent)]
    Database(anyhow::Error),

    /// Failed to enforce maximum session limit (couldn't delete oldest session).
    #[error(transparent)]
    EnforceSessionMaximumError(#[from] EnforceSessionMaximumError),

    /// User not found or database error fetching user.
    #[error(transparent)]
    GetUserError(#[from] GetUserError),

    /// Failed to generate JWT access token.
    #[error(transparent)]
    InvalidJwt(#[from] InvalidJwt),
}

#[cfg(feature = "zerver")]
impl FromStr for CreateSession {
    type Err = InvalidCreateSession;

    /// Parses a user ID string into a `CreateSession` request.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let request: CreateSession = "550e8400-e29b-41d4-a716-446655440000".parse()?;
    /// ```
    fn from_str(s: &str) -> Result<Self, InvalidCreateSession> {
        Ok(Self {
            user_id: Uuid::try_parse(s)?,
        })
    }
}

#[cfg(feature = "zerver")]
/// Request to create a new session for a user.
///
/// This is typically used after successful authentication or by administrators
/// for user impersonation. The service will:
///
/// 1. Verify the user exists
/// 2. Enforce session maximum (5 per user)
/// 3. Generate access and refresh tokens
/// 4. Return a complete [`Session`](super::Session)
///
/// # Security
///
/// This operation should only be performed after verifying the user's identity
/// (password check, SSO, admin authorization, etc.). Creating a session grants
/// full API access for that user.
///
/// # Example
///
/// ```rust,ignore
/// use zwipe::domain::auth::models::session::create_session::CreateSession;
/// use uuid::Uuid;
///
/// // After password verification
/// let request = CreateSession { user_id };
/// let session = session_service.create_session(request).await?;
///
/// // Or use From conversion
/// let request = CreateSession::from(user_id);
/// ```
#[derive(Debug, Clone)]
pub struct CreateSession {
    /// The ID of the user for whom to create a session.
    pub user_id: Uuid,
}

#[cfg(feature = "zerver")]
impl From<Uuid> for CreateSession {
    /// Creates a `CreateSession` request from a user ID.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let request = CreateSession::from(user_id);
    /// ```
    fn from(value: Uuid) -> Self {
        Self { user_id: value }
    }
}
