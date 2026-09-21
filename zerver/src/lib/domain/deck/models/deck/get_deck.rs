//! Get complete deck operation (profile + all cards).
//!
//! Retrieves deck metadata and all cards in the deck with full card data.

use crate::domain::{
    card::requests::{get_card::GetCardError, get_card_profile::GetCardProfileError},
    deck::models::{
        deck::get_deck_profile::GetDeckProfileError, deck_card::get_deck_card::GetDeckCardError,
    },
};
use thiserror::Error;

/// Errors that can occur when retrieving a complete deck.
///
/// Aggregates errors from multiple sub-operations:
/// - Getting deck profile
/// - Getting deck cards
/// - Getting card profiles
/// - Getting card data
#[derive(Debug, Error)]
pub enum GetDeckError {
    /// Error retrieving deck profile (metadata).
    #[error(transparent)]
    GetDeckProfileError(#[from] GetDeckProfileError),
    /// Error retrieving deck_card entries.
    #[error(transparent)]
    GetDeckCardError(#[from] GetDeckCardError),
    /// Error retrieving card profile (application metadata).
    #[error(transparent)]
    GetCardProfileError(#[from] GetCardProfileError),
    /// Error retrieving card data (Scryfall data).
    #[error(transparent)]
    GetCardError(#[from] GetCardError),
}
