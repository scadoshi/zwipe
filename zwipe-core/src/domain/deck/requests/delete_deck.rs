//! Delete deck operation.

use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur while constructing a [`DeleteDeck`] request.
#[derive(Debug, Error)]
pub enum InvalidDeleteDeck {
    /// Invalid user ID format.
    #[error(transparent)]
    UserId(uuid::Error),
    /// Invalid deck ID format.
    #[error(transparent)]
    DeckId(uuid::Error),
}

/// Request to delete a deck.
#[derive(Debug, Clone)]
pub struct DeleteDeck {
    /// Requesting user (for authorization).
    pub user_id: Uuid,
    /// Deck to delete.
    pub deck_id: Uuid,
}

impl DeleteDeck {
    /// Creates a new deck deletion request with validation.
    pub fn new(user_id: Uuid, deck_id: &str) -> Result<Self, InvalidDeleteDeck> {
        let deck_id = Uuid::try_parse(deck_id.trim()).map_err(InvalidDeleteDeck::DeckId)?;
        Ok(Self { user_id, deck_id })
    }
}
