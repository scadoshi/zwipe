//! Add card to deck operation.

use crate::domain::deck::{InvalidQuantity, Quantity};
use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur while constructing a [`CreateDeckCard`] request.
#[derive(Debug, Error)]
pub enum InvalidCreateDeckCard {
    /// Invalid deck ID format.
    #[error(transparent)]
    DeckId(uuid::Error),
    /// Invalid card ID format.
    #[error(transparent)]
    ScryfallDataId(uuid::Error),
    /// Quantity is invalid.
    #[error(transparent)]
    Quantity(InvalidQuantity),
}

impl From<InvalidQuantity> for InvalidCreateDeckCard {
    fn from(value: InvalidQuantity) -> Self {
        Self::Quantity(value)
    }
}

/// Request to add a card to a deck.
#[derive(Debug, Clone)]
pub struct CreateDeckCard {
    /// Requesting user (for authorization).
    pub user_id: Uuid,
    /// Deck to add card to.
    pub deck_id: Uuid,
    /// Card to add (Scryfall data ID).
    pub scryfall_data_id: Uuid,
    /// How many copies.
    pub quantity: Quantity,
    /// Whether this card is on the maybeboard.
    pub maybeboard: bool,
    /// Whether the requesting user's email is verified.
    pub email_verified: bool,
}

impl CreateDeckCard {
    /// Creates a new deck card addition request with validation.
    pub fn new(
        user_id: Uuid,
        deck_id: &str,
        scryfall_data_id: &str,
        quantity: i32,
        maybeboard: Option<bool>,
        email_verified: bool,
    ) -> Result<Self, InvalidCreateDeckCard> {
        let deck_id = Uuid::try_parse(deck_id).map_err(InvalidCreateDeckCard::DeckId)?;
        let scryfall_data_id =
            Uuid::try_parse(scryfall_data_id).map_err(InvalidCreateDeckCard::ScryfallDataId)?;
        let quantity = Quantity::new(quantity)?;

        Ok(Self {
            deck_id,
            scryfall_data_id,
            quantity,
            maybeboard: maybeboard.unwrap_or(false),
            user_id,
            email_verified,
        })
    }
}
