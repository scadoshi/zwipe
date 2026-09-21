//! Clear deck suppressions operation.
//!
//! Re-exported from `zwipe_core`. Service-layer error type remains here.

use thiserror::Error;

/// Errors that can occur while clearing a deck's suppression set.
#[derive(Debug, Error)]
pub enum ClearDeckSuppressionsError {
    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),
    /// Requesting user doesn't own this deck.
    #[error("deck does not belong to requesting user")]
    Forbidden,
}
