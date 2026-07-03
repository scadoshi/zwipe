//! Skip deck card operation (single durable suppression).
//!
//! Re-exported from `zwipe_core`. Service-layer error type remains here.

#[cfg(feature = "zerver")]
use thiserror::Error;

/// Errors that can occur while skipping or unskipping a single deck card.
#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum SkipDeckCardError {
    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),
    /// Requesting user doesn't own this deck.
    #[error("deck does not belong to requesting user")]
    Forbidden,
}
