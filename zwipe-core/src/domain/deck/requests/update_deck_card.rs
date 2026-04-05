//! Update deck card operation (quantity delta and/or board change).

use crate::domain::deck::{Board, InvalidUpdateQuanity, UpdateQuantity};
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
    /// Invalid new printing ID format.
    #[error(transparent)]
    NewScryfallDataId(uuid::Error),
    /// Update quantity is zero (no-op not allowed).
    #[error(transparent)]
    UpdateQuantity(InvalidUpdateQuanity),
    /// No fields provided to update.
    #[error("at least one of update_quantity, board, or scryfall_data_id must be provided")]
    NothingToUpdate,
}

impl From<InvalidUpdateQuanity> for InvalidUpdateDeckCard {
    fn from(value: InvalidUpdateQuanity) -> Self {
        Self::UpdateQuantity(value)
    }
}

/// Request to update a card in a deck.
///
/// Supports updating quantity (delta), board, or both.
/// At least one field must be provided.
#[derive(Debug, Clone)]
pub struct UpdateDeckCard {
    /// Requesting user (for authorization).
    pub user_id: Uuid,
    /// Deck containing the card.
    pub deck_id: Uuid,
    /// Card to update (Scryfall data ID).
    pub scryfall_data_id: Uuid,
    /// Delta value (positive = add, negative = remove). `None` = no quantity change.
    pub update_quantity: Option<UpdateQuantity>,
    /// Move card to this board. `None` = no change.
    pub board: Option<Board>,
    /// Change the selected printing to this Scryfall data ID. `None` = no change.
    pub new_scryfall_data_id: Option<Uuid>,
}

impl UpdateDeckCard {
    /// Creates a new deck card update request with validation.
    ///
    /// At least one of `update_quantity`, `board`, or `new_scryfall_data_id` must be `Some`.
    pub fn new(
        user_id: Uuid,
        deck_id: &str,
        scryfall_data_id: &str,
        update_quantity: Option<i32>,
        board: Option<Board>,
        new_scryfall_data_id: Option<&str>,
    ) -> Result<Self, InvalidUpdateDeckCard> {
        let deck_id = Uuid::try_parse(deck_id).map_err(InvalidUpdateDeckCard::DeckId)?;
        let scryfall_data_id =
            Uuid::try_parse(scryfall_data_id).map_err(InvalidUpdateDeckCard::ScryfallDataId)?;

        let update_quantity = update_quantity
            .map(UpdateQuantity::new)
            .transpose()?;

        let new_scryfall_data_id = new_scryfall_data_id
            .map(|s| Uuid::try_parse(s).map_err(InvalidUpdateDeckCard::NewScryfallDataId))
            .transpose()?;

        if update_quantity.is_none() && board.is_none() && new_scryfall_data_id.is_none() {
            return Err(InvalidUpdateDeckCard::NothingToUpdate);
        }

        Ok(Self {
            user_id,
            deck_id,
            scryfall_data_id,
            update_quantity,
            board,
            new_scryfall_data_id,
        })
    }
}
