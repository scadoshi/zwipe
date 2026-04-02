//! Deck card import operation.
//!
//! Re-exported from `zwipe_core`. Service-layer error type remains here.

pub use zwipe_core::domain::deck::requests::import_deck_cards::*;

#[cfg(feature = "zerver")]
use crate::domain::deck::models::deck::get_deck_profile::GetDeckProfileError;
#[cfg(feature = "zerver")]
use thiserror::Error;

/// Errors that can occur during deck card import.
#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum ImportDeckCardsError {
    /// Requesting user doesn't own this deck.
    #[error("deck does not belong to requesting user")]
    Forbidden,
    /// Deck not found or inaccessible.
    #[error(transparent)]
    DeckNotFound(#[from] GetDeckProfileError),
    /// Import would exceed the maximum number of cards per deck.
    #[error("card limit reached")]
    LimitReached,
    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),
}
