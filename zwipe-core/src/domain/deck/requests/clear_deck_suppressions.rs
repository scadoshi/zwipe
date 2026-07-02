//! Clear deck suppressions operation.

use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur while constructing a [`ClearDeckSuppressions`] request.
#[derive(Debug, Error)]
pub enum InvalidClearDeckSuppressions {
    /// Invalid deck ID format.
    #[error(transparent)]
    DeckId(uuid::Error),
}

/// Request to clear a deck's suppression set (skipped/removed cards).
#[derive(Debug, Clone)]
pub struct ClearDeckSuppressions {
    /// Requesting user (for authorization).
    pub user_id: Uuid,
    /// Deck whose suppressions to clear.
    pub deck_id: Uuid,
}

impl ClearDeckSuppressions {
    /// Creates a new clear-suppressions request with validation.
    pub fn new(user_id: Uuid, deck_id: &str) -> Result<Self, InvalidClearDeckSuppressions> {
        let deck_id =
            Uuid::try_parse(deck_id.trim()).map_err(InvalidClearDeckSuppressions::DeckId)?;
        Ok(Self { user_id, deck_id })
    }
}
