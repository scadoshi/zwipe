#[cfg(feature = "zerver")]
use crate::domain::deck::models::deck::get_deck_profile::GetDeckProfileError;
use crate::domain::deck::models::deck_card::quantity::{InvalidQuantity, Quantity};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum InvalidCreateDeckCard {
    #[error(transparent)]
    DeckId(uuid::Error),
    #[error(transparent)]
    ScryfallDataId(uuid::Error),
    #[error(transparent)]
    Quantity(InvalidQuantity),
}

impl From<InvalidQuantity> for InvalidCreateDeckCard {
    fn from(value: InvalidQuantity) -> Self {
        Self::Quantity(value)
    }
}

#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum CreateDeckCardError {
    #[error("card and deck combination already exists")]
    Duplicate,
    #[error("deck card created but database returned invalid object {0}")]
    DeckCardFromDb(anyhow::Error),
    #[error(transparent)]
    Database(anyhow::Error),
    #[error(transparent)]
    GetDeckProfileError(#[from] GetDeckProfileError),
    #[error("deck does not belong to requesting user")]
    Forbidden,
}

#[derive(Debug, Clone)]
pub struct CreateDeckCard {
    pub user_id: Uuid,
    pub deck_id: Uuid,
    pub scryfall_data_id: Uuid,
    pub quantity: Quantity,
}

impl CreateDeckCard {
    pub fn new(
        user_id: Uuid,
        deck_id: &str,
        scryfall_data_id: &str,
        quantity: i32,
    ) -> Result<Self, InvalidCreateDeckCard> {
        let deck_id = Uuid::try_parse(deck_id).map_err(InvalidCreateDeckCard::DeckId)?;
        let scryfall_data_id =
            Uuid::try_parse(scryfall_data_id).map_err(InvalidCreateDeckCard::ScryfallDataId)?;
        let quantity = Quantity::new(quantity)?;

        Ok(Self {
            deck_id,
            scryfall_data_id,
            quantity,
            user_id,
        })
    }
}
