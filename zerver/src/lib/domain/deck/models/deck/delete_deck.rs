//! Delete deck operation.
//!
//! Re-exported from `zwipe_core`. Service-layer error type remains here.


#[cfg(feature = "zerver")]
use thiserror::Error;

/// Errors that can occur during deck deletion execution.
#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum DeleteDeckError {
    /// Deck ID doesn't exist.
    #[error("deck not found")]
    NotFound,
    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),
    /// Requesting user doesn't own this deck.
    #[error("deck does not belong to requesting user")]
    Forbidden,
}
