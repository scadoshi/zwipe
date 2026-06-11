//! Deck-aware card search operation (exclusion + synergy ordering).
//!
//! Service-layer error type only — request/response shapes are the shared
//! `CardFilter` / `Card` from zwipe_core.

#[cfg(feature = "zerver")]
use crate::domain::{
    card::models::search_card::error::SearchCardsError,
    deck::models::deck::get_deck_profile::GetDeckProfileError,
};
#[cfg(feature = "zerver")]
use thiserror::Error;

/// Errors from the deck-aware card search (exclusion + synergy ordering).
#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum SearchDeckCardsError {
    /// Deck lookup failed (not found / forbidden / database).
    #[error(transparent)]
    GetDeckProfile(#[from] GetDeckProfileError),
    /// Underlying card search failed.
    #[error(transparent)]
    SearchCards(#[from] SearchCardsError),
    /// Other database failure (deck cards, slot resolution).
    #[error(transparent)]
    Database(anyhow::Error),
}
