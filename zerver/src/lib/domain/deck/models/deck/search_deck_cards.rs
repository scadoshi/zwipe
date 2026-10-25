//! Deck-aware card search operation (exclusion + synergy ordering).
//!
//! The request is the shared `CardQuery` from zwipe_core; this module holds
//! the service's result and error types.

use thiserror::Error;
use zwipe_core::domain::card::Card;

use crate::domain::card::models::search_card::error::SearchCardsError;
use crate::domain::deck::models::deck::get_deck_profile::GetDeckProfileError;

/// One page of the deck-aware card search.
#[derive(Debug)]
pub struct DeckCardServe {
    /// The cards, in serve order.
    pub cards: Vec<Card>,
    /// Synergy was requested but the commander's cache wasn't ready, so the
    /// page came from the full pool.
    pub synergy_warming: bool,
    /// Index into `cards` where related cards begin: the synergy pool ran out
    /// and the page was filled with cards like the deck. `None` when every
    /// card on the page is from the pool.
    pub related_from: Option<usize>,
}

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
