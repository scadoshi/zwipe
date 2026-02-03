//! Delete deck operation.
//!
//! Permanently removes a deck and all its cards (cascading delete).
//! Only the deck owner can delete their deck.
//!
//! # Cascading Behavior
//!
//! Deleting a deck also deletes:
//! - All deck_card entries (cards in the deck)
//! - Deck profile metadata
//!
//! This is a destructive operation with no undo.

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

/// Errors that can occur during deck deletion execution.
#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum DeleteDeckError {
    /// Deck ID doesn't exist.
    #[error("deck not found")]
    NotFound,
    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),
    /// Requesting user doesn't own this deck.
    #[error("deck does not belong to requesting user")]
    Forbidden,
}

/// Request to delete a deck.
///
/// Deletes the deck and all associated cards (cascading delete).
/// Authorization check ensures only the owner can delete.
///
/// # Example
///
/// ```rust,ignore
/// let delete = DeleteDeck::new(user_id, "deck-uuid-string")?;
/// deck_service.delete_deck(&delete).await?;
/// ```
#[derive(Debug, Clone)]
pub struct DeleteDeck {
    /// Requesting user (for authorization).
    pub user_id: Uuid,
    /// Deck to delete.
    pub deck_id: Uuid,
}

impl DeleteDeck {
    /// Creates a new deck deletion request with validation.
    ///
    /// # Parameters
    ///
    /// - `user_id`: Requesting user's ID
    /// - `deck_id`: Deck ID as string (will be parsed and validated)
    ///
    /// # Errors
    ///
    /// Returns [`InvalidDeleteDeck`] if deck_id is not a valid UUID.
    pub fn new(user_id: Uuid, deck_id: &str) -> Result<Self, InvalidDeleteDeck> {
        let deck_id = Uuid::try_parse(deck_id.trim()).map_err(InvalidDeleteDeck::DeckId)?;
        Ok(Self { user_id, deck_id })
    }
}
