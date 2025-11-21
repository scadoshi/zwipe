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
    CardProfileId(uuid::Error),
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
    GetDeckProfileError(GetDeckProfileError),
    #[error("deck does not belong to requesting user")]
    Forbidden,
}

#[cfg(feature = "zerver")]
impl From<GetDeckProfileError> for CreateDeckCardError {
    fn from(value: GetDeckProfileError) -> Self {
        Self::GetDeckProfileError(value)
    }
}

#[derive(Debug, Clone)]
pub struct CreateDeckCard {
    pub user_id: Uuid,
    pub deck_id: Uuid,
    pub card_profile_id: Uuid,
    pub quantity: Quantity,
}

impl CreateDeckCard {
    pub fn new(
        user_id: Uuid,
        deck_id: &str,
        card_profile_id: &str,
        quantity: i32,
    ) -> Result<Self, InvalidCreateDeckCard> {
        let deck_id = Uuid::try_parse(deck_id).map_err(InvalidCreateDeckCard::DeckId)?;
        let card_profile_id =
            Uuid::try_parse(card_profile_id).map_err(InvalidCreateDeckCard::CardProfileId)?;
        let quantity = Quantity::new(quantity)?;

        Ok(Self {
            deck_id,
            card_profile_id,
            quantity,
            user_id,
        })
    }
}
