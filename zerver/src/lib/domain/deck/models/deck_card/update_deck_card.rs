#[cfg(feature = "zerver")]
use crate::domain::deck::models::deck::get_deck_profile::GetDeckProfileError;
use crate::domain::deck::models::deck_card::quantity::{InvalidUpdateQuanity, UpdateQuantity};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum InvalidUpdateDeckCard {
    #[error(transparent)]
    DeckId(uuid::Error),
    #[error(transparent)]
    ScryfallDataId(uuid::Error),
    #[error(transparent)]
    UpdateQuantity(InvalidUpdateQuanity),
}

impl From<InvalidUpdateQuanity> for InvalidUpdateDeckCard {
    fn from(value: InvalidUpdateQuanity) -> Self {
        Self::UpdateQuantity(value)
    }
}

#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum UpdateDeckCardError {
    #[error("deck card not found")]
    NotFound,
    #[error("resulting quantity must remain greater than 0")]
    InvalidResultingQuantity,
    #[error(transparent)]
    Database(anyhow::Error),
    #[error("deck card updated but database returned invalid object: {0}")]
    DeckCardFromDb(anyhow::Error),
    #[error(transparent)]
    GetDeckProfileError(#[from] GetDeckProfileError),
    #[error("deck does not belong to requesting user")]
    Forbidden,
}

#[derive(Debug, Clone)]
pub struct UpdateDeckCard {
    pub user_id: Uuid,
    pub deck_id: Uuid,
    pub scryfall_data_id: Uuid,
    pub update_quantity: UpdateQuantity,
}

impl UpdateDeckCard {
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
