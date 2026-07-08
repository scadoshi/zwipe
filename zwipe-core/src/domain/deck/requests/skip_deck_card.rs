//! Skip deck card operation (single durable suppression).

use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur while constructing a [`SkipDeckCard`] request.
#[derive(Debug, Error)]
pub enum InvalidSkipDeckCard {
    /// Invalid deck ID format.
    #[error(transparent)]
    DeckId(uuid::Error),
    /// Invalid oracle ID format.
    #[error(transparent)]
    OracleId(uuid::Error),
}

/// Request to skip (suppress) or unskip a single card for a deck.
#[derive(Debug, Clone)]
pub struct SkipDeckCard {
    /// Requesting user (for authorization).
    pub user_id: Uuid,
    /// Deck the skip applies to.
    pub deck_id: Uuid,
    /// Oracle id of the card (covers all printings).
    pub oracle_id: Uuid,
}

impl SkipDeckCard {
    /// Creates a new skip request with validation.
    pub fn new(user_id: Uuid, deck_id: &str, oracle_id: Uuid) -> Result<Self, InvalidSkipDeckCard> {
        let deck_id = Uuid::try_parse(deck_id.trim()).map_err(InvalidSkipDeckCard::DeckId)?;
        Ok(Self {
            user_id,
            deck_id,
            oracle_id,
        })
    }

    /// Creates a new skip request parsing both ids from path segments.
    pub fn from_path(
        user_id: Uuid,
        deck_id: &str,
        oracle_id: &str,
    ) -> Result<Self, InvalidSkipDeckCard> {
        let oracle_id = Uuid::try_parse(oracle_id.trim()).map_err(InvalidSkipDeckCard::OracleId)?;
        Self::new(user_id, deck_id, oracle_id)
    }
}
