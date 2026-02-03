//! Delete card from deck operation.
//!
//! Removes a card entry from a deck entirely (all copies).
//! Only the deck owner can delete cards from their deck.

use thiserror::Error;
use uuid::Uuid;

#[cfg(feature = "zerver")]
use crate::domain::deck::models::deck::get_deck_profile::GetDeckProfileError;

/// Errors that can occur during deck card deletion execution.
#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum DeleteDeckCardError {
    /// Card doesn't exist in this deck.
    #[error("deck card not found")]
    NotFound,
    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),
    /// Error retrieving deck for authorization check.
    #[error(transparent)]
    GetDeckProfileError(#[from] GetDeckProfileError),
    /// Requesting user doesn't own this deck.
    #[error("deck does not belong to requesting user")]
    Forbidden,
}


/// Errors that occur while constructing a [`DeleteDeckCard`] request.
#[derive(Debug, Error)]
pub enum InvalidDeleteDeckCard {
    /// Invalid deck ID format.
    #[error(transparent)]
    DeckId(uuid::Error),
    /// Invalid card ID format.
    #[error(transparent)]
    ScryfallDataId(uuid::Error),
}


/// Request to delete a card from a deck (removes all copies).
///
/// # Authorization
///
/// Only the deck owner can delete cards from their deck.
///
/// # Example
///
/// ```rust,ignore
/// let delete = DeleteDeckCard::new(user_id, "deck-uuid", "card-uuid")?;
/// deck_service.delete_deck_card(&delete).await?;
/// ```
#[derive(Debug, Clone)]
pub struct DeleteDeckCard {
    /// Requesting user (for authorization).
    pub user_id: Uuid,
    /// Deck containing the card.
    pub deck_id: Uuid,
    /// Card to remove (Scryfall data ID).
    pub scryfall_data_id: Uuid,
}

impl DeleteDeckCard {
    /// Creates a new deck card deletion request with validation.
    ///
    /// # Parameters
    ///
    /// - `user_id`: Requesting user's ID
    /// - `deck_id`: Deck ID as string (will be parsed)
    /// - `scryfall_data_id`: Card ID as string (will be parsed)
    ///
    /// # Errors
    ///
    /// Returns [`InvalidDeleteDeckCard`] if:
    /// - Deck ID is not a valid UUID
    /// - Card ID is not a valid UUID
    pub fn new(
        user_id: Uuid,
        deck_id: &str,
        scryfall_data_id: &str,
    ) -> Result<Self, InvalidDeleteDeckCard> {
        let deck_id = Uuid::try_parse(deck_id).map_err(InvalidDeleteDeckCard::DeckId)?;
        let scryfall_data_id =
            Uuid::try_parse(scryfall_data_id).map_err(InvalidDeleteDeckCard::ScryfallDataId)?;
        Ok(Self {
            user_id,
            deck_id,
            scryfall_data_id,
        })
    }
}
