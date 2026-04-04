//! Add card to deck operation.
//!
//! Re-exported from `zwipe_core`. Service-layer error type remains here.


#[cfg(feature = "zerver")]
use crate::domain::deck::models::deck::get_deck_profile::GetDeckProfileError;
#[cfg(feature = "zerver")]
use thiserror::Error;

/// Errors that can occur during deck card creation execution.
#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum CreateDeckCardError {
    /// Card already exists in this deck (use update to change quantity).
    #[error("card and deck combination already exists")]
    Duplicate,
    /// Cannot add the deck's commander as a regular card.
    #[error("card is this deck's commander")]
    IsCommander,
    /// Deck has reached the maximum number of cards.
    #[error("card limit reached")]
    LimitReached,
    /// Database returned invalid data after creation.
    #[error("deck card created but database returned invalid object {0}")]
    DeckCardFromDb(anyhow::Error),
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
