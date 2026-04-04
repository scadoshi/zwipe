//! Update deck card quantity operation.
//!
//! Re-exported from `zwipe_core`. Service-layer error type remains here.


#[cfg(feature = "zerver")]
use crate::domain::deck::models::deck::get_deck_profile::GetDeckProfileError;
#[cfg(feature = "zerver")]
use thiserror::Error;

/// Errors that can occur during deck card update execution.
#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum UpdateDeckCardError {
    /// Card doesn't exist in this deck (use create to add new cards).
    #[error("deck card not found")]
    NotFound,
    /// Delta would result in quantity ≤ 0 (use delete to remove card entirely).
    #[error("resulting quantity cannot be zero or less")]
    QuantityUnderflow,
    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),
    /// Database returned invalid data after update.
    #[error("deck card updated but database returned invalid object: {0}")]
    DeckCardFromDb(anyhow::Error),
    /// Error retrieving deck for authorization check.
    #[error(transparent)]
    GetDeckProfileError(#[from] GetDeckProfileError),
    /// Requesting user doesn't own this deck.
    #[error("deck does not belong to requesting user")]
    Forbidden,
}
