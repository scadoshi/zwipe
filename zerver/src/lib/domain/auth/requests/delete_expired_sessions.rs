//! Expired session cleanup operation.
//!
//! Removes sessions that have exceeded their 14-day refresh token lifespan.
//! This is a maintenance operation typically run periodically (e.g., daily cron job).
//!
//! # Why Clean Up?
//!
//! Expired sessions cannot be used (refresh will fail), but they still:
//! - Consume database storage
//! - Slow down session queries
//! - Make it harder to analyze active user counts
//!
//! # When to Run
//!
//! - **Periodic**: Daily background job (off-peak hours)
//! - **On-Demand**: After major events (e.g., security breach)
//! - **Startup**: During application initialization
//!
//! # Example
//!
//! ```rust,ignore
//! // Delete all expired sessions (no parameters needed)
//! session_service.delete_expired_sessions().await?;
//! ```

use thiserror::Error;

/// Errors that can occur while deleting expired sessions.
#[derive(Debug, Error)]
pub enum DeleteExpiredSessionsError {
    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),
}
