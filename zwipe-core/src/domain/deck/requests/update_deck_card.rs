//! Update deck card quantity operation.

use crate::domain::deck::{InvalidUpdateQuanity, UpdateQuantity};
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

/// Request to update card quantity in a deck (add or remove copies).
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
