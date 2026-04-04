//! Update deck profile operation.
//!
//! Re-exported from `zwipe_core`. Service-layer error type remains here.


#[cfg(feature = "zerver")]
use crate::domain::deck::models::deck::get_deck_profile::GetDeckProfileError;
#[cfg(feature = "zerver")]
use thiserror::Error;

/// Errors that can occur during deck profile update execution.
#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum UpdateDeckProfileError {
    /// User already has another deck with this name.
    #[error("deck with name and user id combination already exists")]
    Duplicate,
    /// Deck ID doesn't exist.
    #[error("deck not found")]
    NotFound,
    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),
    /// Database returned invalid data after update.
    #[error("deck updated but database returned invalid object: {0}")]
    DeckFromDb(anyhow::Error),
    /// Error retrieving deck for authorization check.
    #[error(transparent)]
    GetDeckProfileError(#[from] GetDeckProfileError),
    /// Requesting user doesn't own this deck.
    #[error("deck does not belong to requesting user")]
    Forbidden,
}
