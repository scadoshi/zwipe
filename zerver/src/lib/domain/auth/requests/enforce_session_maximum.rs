//! Session limit enforcement (maximum 5 concurrent sessions per user).
//!
//! Automatically revokes the oldest session when a user exceeds the 5-session limit.
//! This prevents session accumulation while allowing reasonable multi-device usage.
//!
//! # Why Limit Sessions?
//!
//! - **Security**: Reduces attack surface if credentials are compromised
//! - **Performance**: Prevents unbounded session table growth
//! - **User Experience**: Encourages logout on unused devices
//!
//! # How It Works
//!
//! 1. Count user's current sessions
//! 2. If count >= 5, delete oldest session (by creation time)
//! 3. Allow new session to be created
//!
//! # Called During
//!
//! - User login (before creating new session)
//! - Token refresh (checked but usually no-op since session exists)
//!
//! # Example
//!
//! ```rust,ignore
//! // Called automatically during session creation
//! session_service.enforce_session_maximum(user_id).await?;
//! session_service.create_session(user_id).await?;
//! ```

use thiserror::Error;

/// Errors that can occur while enforcing session maximum.
#[derive(Debug, Error)]
pub enum EnforceSessionMaximumError {
    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),
}
