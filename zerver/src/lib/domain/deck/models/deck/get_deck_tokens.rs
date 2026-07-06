//! Get tokens produced by all cards in a deck.
//!
//! Extracts token Scryfall IDs from each card's `all_parts` field
//! and resolves them to full card objects.

#[cfg(feature = "zerver")]
use crate::domain::{
    card::requests::get_card::GetCardError, deck::models::deck::get_deck::GetDeckError,
};
#[cfg(feature = "zerver")]
use thiserror::Error;

/// Errors that can occur when retrieving tokens for a deck.
#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum GetDeckTokensError {
    /// Error retrieving the deck itself.
    #[error(transparent)]
    GetDeckError(#[from] GetDeckError),
    /// Error retrieving token card data.
    #[error(transparent)]
    GetCardError(#[from] GetCardError),
}
