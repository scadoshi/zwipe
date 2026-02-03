//! Session management models and operations.
//!
//! This module handles the creation, refresh, and lifecycle management of user sessions.
//! Sessions use a dual-token approach for security:
//!
//! - **Access Token**: Short-lived JWT (24h) for authenticating API requests
//! - **Refresh Token**: Long-lived token (14d) for obtaining new access tokens
//!
//! # Session Lifecycle
//!
//! 1. **Creation**: User authenticates → session created with both tokens
//! 2. **Usage**: Access token authenticates requests until expiry
//! 3. **Refresh**: When access token expires, refresh token obtains new access token
//! 4. **Rotation**: Refresh tokens are single-use and rotate on each refresh
//! 5. **Expiry**: After 14 days, user must re-authenticate
//!
//! # Security Features
//!
//! - **Session Limits**: Maximum 5 concurrent sessions per user
//! - **Auto-Cleanup**: Expired sessions automatically deleted
//! - **Refresh Token Rotation**: Single-use refresh tokens prevent replay attacks
//! - **SHA-256 Hashing**: Refresh tokens hashed in database
//!
//! # Submodules
//!
//! - [`create_session`]: Create new session for a user
//! - [`refresh_session`]: Exchange refresh token for new access token
//! - [`revoke_sessions`]: Delete all user sessions (logout)
//! - [`enforce_session_maximum`]: Enforce max session limit per user
//! - [`delete_expired_sessions`]: Cleanup expired sessions

pub mod create_session;
pub mod delete_expired_sessions;
pub mod enforce_session_maximum;
pub mod refresh_session;
pub mod revoke_sessions;

use crate::domain::auth::models::access_token::AccessToken;
use crate::domain::auth::models::refresh_token::RefreshToken;
use crate::domain::user::models::User;
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Maximum number of concurrent sessions allowed per user.
///
/// When this limit is exceeded, the oldest session is automatically revoked.
/// This prevents unlimited session accumulation and limits the attack surface
/// if refresh tokens are compromised.
///
/// Users can have sessions across multiple devices (e.g., phone, tablet, desktop,
/// browser, etc.) up to this limit.
pub const MAXIMUM_SESSION_COUNT: u8 = 5;

/// A successful authentication response containing user data and tokens.
///
/// This is the primary response type for authentication operations (login, registration,
/// refresh). It includes everything needed for the client to authenticate subsequent
/// API requests.
///
/// # Token Lifecycle
///
/// - **Access Token**: Valid for 24 hours, used in `Authorization: Bearer <token>` header
/// - **Refresh Token**: Valid for 14 days, used to obtain new access tokens
///
/// # Usage Pattern
///
/// 1. Client receives session from login/register
/// 2. Client stores both tokens securely
/// 3. Client uses access token for API requests
/// 4. When access token expires, client uses refresh token to get new session
/// 5. After 14 days, refresh token expires and user must re-authenticate
///
/// # Example
///
/// ```rust,ignore
/// use zwipe::domain::auth::models::session::Session;
///
/// // After successful authentication
/// let session: Session = auth_service.authenticate(request).await?;
///
/// // Client uses these for API requests
/// println!("User ID: {}", session.user.id);
/// println!("Access Token: {}", session.access_token);
/// println!("Refresh Token (expires: {})", session.refresh_token.expires_at);
///
/// // Check if session needs refresh
/// if session.is_expired() {
///     // User must re-authenticate
/// }
/// ```
#[derive(Debug, Clone, Serialize, PartialEq, Deserialize)]
pub struct Session {
    /// The authenticated user's public profile information.
    pub user: User,

    /// JWT access token for authenticating API requests (24h expiry).
    pub access_token: AccessToken,

    /// Long-lived refresh token for obtaining new access tokens (14d expiry).
    pub refresh_token: RefreshToken,
}

impl Session {
    /// Creates a new session with the given user and tokens.
    ///
    /// Typically called by authentication services after successful login or registration.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let session = Session::new(user, access_token, refresh_token);
    /// ```
    #[cfg(feature = "zerver")]
    pub fn new(user: User, access_token: AccessToken, refresh_token: RefreshToken) -> Self {
        Session {
            user,
            access_token,
            refresh_token,
        }
    }

    /// Checks if the refresh token has expired.
    ///
    /// When `true`, the user must re-authenticate as the refresh token can no
    /// longer be used to obtain new access tokens.
    ///
    /// # Returns
    ///
    /// - `true` if the refresh token expiry time has passed
    /// - `false` if the refresh token is still valid
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// if session.is_expired() {
    ///     return Err(AuthError::SessionExpired);
    /// }
    /// ```
    pub fn is_expired(&self) -> bool {
        self.refresh_token.expires_at < Utc::now().naive_local()
    }
}
