//! Create deck profile operation.
//!
//! Re-exported from `zwipe_core`. Service-layer error type remains here.

pub use zwipe_core::domain::deck::requests::create_deck_profile::*;

#[cfg(feature = "zerver")]
use thiserror::Error;

/// Errors that can occur during deck profile creation execution.
#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum CreateDeckProfileError {
    /// User already has a deck with this name.
    #[error("deck with name and user id combination already exists")]
    Duplicate,
    /// User has reached the maximum number of decks.
    #[error("deck limit reached")]
    LimitReached,
    /// Database returned invalid data after creation.
    #[error("deck created but database returned invalid object {0}")]
    DeckFromDb(anyhow::Error),
    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),
}
