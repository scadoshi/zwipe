use crate::domain::{
    card::models::search_card::error::SearchCardsError,
    deck::models::deck::get_deck_profile::GetDeckProfileError,
};
use thiserror::Error;

/// Errors from the deck-aware card search (exclusion + synergy ordering).
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
