//! Commander maybeboard operations (per-user "maybe this commander" list).
//!
//! Request types are re-exported from `zwipe_core`. Service-layer error type
//! remains here.

#[cfg(feature = "zerver")]
use thiserror::Error;

/// Errors that can occur while reading or mutating a user's commander
/// maybeboard.
#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum CommanderMaybeboardError {
    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),
    /// The oracle id doesn't match any served card.
    #[error("card not found")]
    UnknownCard,
    /// The user's maybeboard is at capacity.
    #[error("commander maybeboard is full")]
    LimitReached,
}
