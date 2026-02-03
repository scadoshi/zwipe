//! Refresh session operation for obtaining new access tokens.
//!
//! This module handles the token refresh flow, allowing clients to obtain new
//! access tokens when their current one expires without requiring re-authentication.
//!
//! # Refresh Flow
//!
//! 1. Client's access token expires (after 24h)
//! 2. Client sends refresh token and user ID
//! 3. Service validates refresh token (exists, not expired, not revoked, matches user)
//! 4. Service deletes old refresh token (single-use)
//! 5. Service creates new session with new access + refresh tokens
//! 6. Client updates stored tokens
//!
//! # Security Features
//!
//! - **Single-Use Tokens**: Refresh tokens are deleted after use (rotation)
//! - **Token Ownership**: Refresh token must belong to requesting user
//! - **Expiry Check**: Tokens expire after 14 days
//! - **Revocation Support**: Tokens can be revoked (logout)
//!
//! # Example
//!
//! ```rust,ignore
//! use zwipe::domain::auth::models::session::refresh_session::RefreshSession;
//!
//! // When access token expires
//! let request = RefreshSession::new(&user_id_str, &refresh_token_value)?;
//! let new_session = session_service.refresh_session(request).await?;
//!
//! // Client now has fresh access token (24h) and refresh token (14d)
//! ```

use crate::domain::auth::models::session::Session;
#[cfg(feature = "zerver")]
use crate::domain::{
    auth::models::{
        access_token::InvalidJwt,
        session::{
            create_session::CreateSessionError, enforce_session_maximum::EnforceSessionMaximumError,
        },
    },
    user::models::get_user::GetUserError,
};
use thiserror::Error;
use uuid::Uuid;

/// Validation errors when constructing a [`RefreshSession`] request.
///
/// Currently only validates that the user ID is a valid UUID.
#[derive(Debug, Error)]
pub enum InvalidRefreshSession {
    /// The provided user ID string is not a valid UUID.
    #[error(transparent)]
    UserId(#[from] uuid::Error),
}

#[cfg(feature = "zerver")]
/// Errors that can occur during session refresh.
///
/// The refresh operation validates the refresh token through multiple checks:
/// existence, ownership, expiry, and revocation status. It then creates a new
/// session, which can also fail.
#[derive(Debug, Error)]
pub enum RefreshSessionError {
    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),

    /// Failed to create new session after validating refresh token.
    #[error(transparent)]
    CreateSessionError(#[from] CreateSessionError),

    /// No refresh token found matching the provided value.
    ///
    /// This could indicate:
    /// - Invalid/corrupted token value
    /// - Token already used (rotation)
    /// - Token manually deleted from database
    #[error("match for given refresh token not found—user attempting: {0}")]
    NotFound(Uuid),

    /// The refresh token has passed its 14-day expiry time.
    ///
    /// User must re-authenticate with username/password to get a new session.
    #[error("given refresh token is expired—user attempting: {0}")]
    Expired(Uuid),

    /// The refresh token was explicitly revoked (user logged out).
    ///
    /// User must re-authenticate to get a new session.
    #[error("given refresh token has been revoked—user attempting: {0}")]
    Revoked(Uuid),

    /// The refresh token belongs to a different user.
    ///
    /// This is a security violation - someone is attempting to use another
    /// user's refresh token. This should be logged and potentially trigger
    /// security alerts.
    #[error("refresh token does not belong to the requesting user—user attempting: {0}")]
    Forbidden(Uuid),

    /// User not found or database error fetching user.
    #[error(transparent)]
    GetUserError(#[from] GetUserError),

    /// Failed to generate new JWT access token.
    #[error(transparent)]
    InvalidJwt(#[from] InvalidJwt),

    /// Failed to enforce maximum session limit.
    #[error(transparent)]
    EnforceSessionMaximumError(#[from] EnforceSessionMaximumError),
}

/// Request to refresh a session using a refresh token.
///
/// The refresh operation validates the token and creates a new session with
/// fresh access and refresh tokens. The old refresh token is deleted (single-use).
///
/// # Security
///
/// - Refresh tokens are single-use - attempting to reuse causes `NotFound` error
/// - Token must belong to the requesting user - mismatches cause `Forbidden` error
/// - Expired tokens cannot be used - user must re-authenticate
///
/// # Example
///
/// ```rust,ignore
/// use zwipe::domain::auth::models::session::refresh_session::RefreshSession;
///
/// // Client's access token expired
/// let request = RefreshSession::new(&user_id, &refresh_token)?;
/// let new_session = session_service.refresh_session(request).await?;
///
/// // Update client storage with new tokens
/// store_access_token(new_session.access_token);
/// store_refresh_token(new_session.refresh_token);
/// ```
#[derive(Debug, Clone)]
pub struct RefreshSession {
    /// The ID of the user requesting the refresh.
    ///
    /// Must match the user ID associated with the refresh token.
    pub user_id: Uuid,

    /// The refresh token value to exchange for new tokens.
    ///
    /// This is the unhashed token value that the client received in the
    /// previous session response.
    pub refresh_token: String,
}

impl RefreshSession {
    /// Creates a new refresh session request.
    ///
    /// # Arguments
    ///
    /// * `user_id` - UUID string of the user requesting refresh
    /// * `refresh_token` - The refresh token value from the previous session
    ///
    /// # Errors
    ///
    /// Returns [`InvalidRefreshSession::UserId`] if the user ID is not a valid UUID.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let request = RefreshSession::new(
    ///     "550e8400-e29b-41d4-a716-446655440000",
    ///     "abc123refreshtoken"
    /// )?;
    /// ```
    pub fn new(user_id: &str, refresh_token: &str) -> Result<Self, InvalidRefreshSession> {
        let user_id = Uuid::try_parse(user_id)?;
        let refresh_token = refresh_token.to_string();
        Ok(Self {
            user_id,
            refresh_token,
        })
    }
}

impl From<&Session> for RefreshSession {
    /// Extracts refresh session request from an existing session.
    ///
    /// Useful for testing or client implementations that want to prepare
    /// the next refresh request from the current session.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let current_session: Session = /* ... */;
    /// let refresh_request = RefreshSession::from(&current_session);
    /// // Later when access token expires:
    /// let new_session = session_service.refresh_session(refresh_request).await?;
    /// ```
    fn from(value: &Session) -> Self {
        Self {
            user_id: value.user.id,
            refresh_token: value.refresh_token.value.clone(),
        }
    }
}
