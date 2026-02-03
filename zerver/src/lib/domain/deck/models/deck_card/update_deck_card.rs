//! Update deck card quantity operation.
//!
//! Increments or decrements the number of copies of a card in a deck.
//! Uses delta values (positive to add, negative to remove copies).

#[cfg(feature = "zerver")]
use crate::domain::deck::models::deck::get_deck_profile::GetDeckProfileError;
use crate::domain::deck::models::deck_card::quantity::{InvalidUpdateQuanity, UpdateQuantity};
use thiserror::Error;
use uuid::Uuid;

/// Errors that occur while constructing an [`UpdateDeckCard`] request.
#[derive(Debug, Error)]
pub enum InvalidUpdateDeckCard {
    /// Invalid deck ID format.
    #[error(transparent)]
    DeckId(uuid::Error),
    /// Invalid card ID format.
    #[error(transparent)]
    ScryfallDataId(uuid::Error),
    /// Update quantity is zero (no-op not allowed).
    #[error(transparent)]
    UpdateQuantity(InvalidUpdateQuanity),
}

impl From<InvalidUpdateQuanity> for InvalidUpdateDeckCard {
    fn from(value: InvalidUpdateQuanity) -> Self {
        Self::UpdateQuantity(value)
    }
}

/// Errors that can occur during deck card update execution.
#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum UpdateDeckCardError {
    /// Card doesn't exist in this deck (use create to add new cards).
    #[error("deck card not found")]
    NotFound,
    /// Delta would result in quantity ≤ 0 (use delete to remove card entirely).
    #[error("resulting quantity must remain greater than 0")]
    InvalidResultingQuantity,
    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),
    /// Database returned invalid data after update.
    #[error("deck card updated but database returned invalid object: {0}")]
    DeckCardFromDb(anyhow::Error),
    /// Error retrieving deck for authorization check.
    #[error(transparent)]
    GetDeckProfileError(#[from] GetDeckProfileError),
    /// Requesting user doesn't own this deck.
    #[error("deck does not belong to requesting user")]
    Forbidden,
}

/// Request to update card quantity in a deck (add or remove copies).
///
/// Uses delta values: positive to add copies, negative to remove copies.
/// Cannot result in quantity ≤ 0 (use delete operation instead).
///
/// # Example
///
/// ```rust,ignore
/// // Add 2 more copies (current: 2 → new: 4)
/// let update = UpdateDeckCard::new(user_id, "deck-uuid", "card-uuid", 2)?;
///
/// // Remove 1 copy (current: 4 → new: 3)
/// let update = UpdateDeckCard::new(user_id, "deck-uuid", "card-uuid", -1)?;
/// ```
#[derive(Debug, Clone)]
pub struct UpdateDeckCard {
    /// Requesting user (for authorization).
    pub user_id: Uuid,
    /// Deck containing the card.
    pub deck_id: Uuid,
    /// Card to update (Scryfall data ID).
    pub scryfall_data_id: Uuid,
    /// Delta value (positive = add, negative = remove).
    pub update_quantity: UpdateQuantity,
}

impl UpdateDeckCard {
    /// Creates a new deck card update request with validation.
    ///
    /// # Parameters
    ///
    /// - `user_id`: Requesting user's ID
    /// - `deck_id`: Deck ID as string (will be parsed)
    /// - `scryfall_data_id`: Card ID as string (will be parsed)
    /// - `update_quantity`: Delta value (positive to add, negative to remove, cannot be 0)
    ///
    /// # Errors
    ///
    /// Returns [`InvalidUpdateDeckCard`] if:
    /// - Deck ID is not a valid UUID
    /// - Card ID is not a valid UUID
    /// - Update quantity is zero (no-op)
    pub fn new(
        user_id: Uuid,
        deck_id: &str,
        scryfall_data_id: &str,
        update_quantity: i32,
    ) -> Result<Self, InvalidUpdateDeckCard> {
        let deck_id = Uuid::try_parse(deck_id).map_err(InvalidUpdateDeckCard::DeckId)?;
        let scryfall_data_id =
            Uuid::try_parse(scryfall_data_id).map_err(InvalidUpdateDeckCard::ScryfallDataId)?;
        let update_quantity = UpdateQuantity::new(update_quantity)?;
        Ok(Self {
            user_id,
            deck_id,
            scryfall_data_id,
            update_quantity,
        })
    }
}
