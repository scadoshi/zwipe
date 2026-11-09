//! Mailing a fresh link for a pending email change.

use thiserror::Error;

/// Errors from re-sending a pending email change's link.
#[derive(Debug, Error)]
pub enum ResendEmailChangeError {
    /// The user has no unexpired email change waiting on its link.
    #[error("no pending email change")]
    NoPendingChange,
    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),
}
