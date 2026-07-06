//! Create deck profile operation.
//!
//! Re-exported from `zwipe_core`. Service-layer error type remains here.

#[cfg(feature = "zerver")]
use thiserror::Error;

/// Errors that can occur during deck profile creation execution.
#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum CreateDeckProfileError {
    /// User already has a deck with this name.
    #[error("deck with name and user id combination already exists")]
    Duplicate,
    /// User has reached the maximum number of decks (verified user, true cap).
    #[error("deck limit reached")]
    LimitReached,
    /// User has reached the unverified deck limit.
    #[error("deck limit reached, verify your email to unlock more")]
    UnverifiedLimitReached,
    /// Database returned invalid data after creation.
    #[error("deck created but database returned invalid object {0}")]
    DeckFromDb(anyhow::Error),
    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),
}
