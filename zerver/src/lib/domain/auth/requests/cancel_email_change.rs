//! Dropping a pending email change before its link is used.

use thiserror::Error;

/// Errors from cancelling a pending email change.
#[derive(Debug, Error)]
pub enum CancelEmailChangeError {
    /// The user has no unexpired email change waiting on its link.
    #[error("no pending email change")]
    NoPendingChange,
    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),
}
