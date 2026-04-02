//! Delete card from deck operation.

use thiserror::Error;
use uuid::Uuid;

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
