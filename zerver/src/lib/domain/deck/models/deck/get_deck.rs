//! Get complete deck operation (profile + all cards).
//!
//! Retrieves deck metadata and all cards in the deck with full card data.

use thiserror::Error;

use crate::domain::card::requests::get_card::GetCardError;
use crate::domain::card::requests::get_card_profile::GetCardProfileError;
use crate::domain::deck::models::deck::get_deck_profile::GetDeckProfileError;
use crate::domain::deck::models::deck_card::get_deck_card::GetDeckCardError;

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
    /// Error retrieving `deck_card` entries.
    #[error(transparent)]
    GetDeckCardError(#[from] GetDeckCardError),
    /// Error retrieving card profile (application metadata).
    #[error(transparent)]
    GetCardProfileError(#[from] GetCardProfileError),
    /// Error retrieving card data (Scryfall data).
    #[error(transparent)]
    GetCardError(#[from] GetCardError),
}
