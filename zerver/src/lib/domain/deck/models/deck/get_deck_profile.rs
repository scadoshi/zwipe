//! Get deck profile operation.
//!
//! Re-exported from `zwipe_core`. Service-layer error type remains here.


#[cfg(feature = "zerver")]
use thiserror::Error;

/// Errors that can occur when retrieving a deck profile.
#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum GetDeckProfileError {
    /// Deck ID doesn't exist in database.
    #[error("deck profile not found")]
    NotFound,
    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),
    /// Database returned invalid data.
    #[error("deck profile found but database returned invalid object: {0}")]
    DeckProfileFromDb(anyhow::Error),
    /// Requesting user doesn't own this deck.
    #[error("deck does not belong to authenticated user")]
    Forbidden,
}
