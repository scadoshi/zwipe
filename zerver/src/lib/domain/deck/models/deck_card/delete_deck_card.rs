//! Delete card from deck operation.
//!
//! Re-exported from `zwipe_core`. Service-layer error type remains here.

pub use zwipe_core::domain::deck::requests::delete_deck_card::*;

#[cfg(feature = "zerver")]
use crate::domain::deck::models::deck::get_deck_profile::GetDeckProfileError;
#[cfg(feature = "zerver")]
use thiserror::Error;

/// Errors that can occur during deck card deletion execution.
#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum DeleteDeckCardError {
    /// Card doesn't exist in this deck.
    #[error("deck card not found")]
    NotFound,
    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),
    /// Error retrieving deck for authorization check.
    #[error(transparent)]
    GetDeckProfileError(#[from] GetDeckProfileError),
    /// Requesting user doesn't own this deck.
    #[error("deck does not belong to requesting user")]
    Forbidden,
}
