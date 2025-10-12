#[cfg(feature = "zerver")]
use crate::domain::deck::models::deck::get_deck::GetDeckProfileError;
use crate::domain::deck::models::deck_card::quantity::{InvalidUpdateQuanity, UpdateQuantity};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum InvalidUpdateDeckCard {
    #[error(transparent)]
    DeckId(uuid::Error),
    #[error(transparent)]
    CardProfileId(uuid::Error),
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
    GetDeckProfileError(GetDeckProfileError),
    #[error("deck does not belong to requesting user")]
    Forbidden,
}

#[cfg(feature = "zerver")]
impl From<GetDeckProfileError> for UpdateDeckCardError {
    fn from(value: GetDeckProfileError) -> Self {
        Self::GetDeckProfileError(value)
    }
}

#[derive(Debug, Clone)]
pub struct UpdateDeckCard {
    pub user_id: Uuid,
    pub deck_id: Uuid,
    pub card_profile_id: Uuid,
    pub update_quantity: UpdateQuantity,
}

impl UpdateDeckCard {
    pub fn new(
        user_id: Uuid,
        deck_id: &str,
        card_profile_id: &str,
        update_quantity: i32,
    ) -> Result<Self, InvalidUpdateDeckCard> {
        let deck_id = Uuid::try_parse(deck_id).map_err(|e| InvalidUpdateDeckCard::DeckId(e))?;
        let card_profile_id = Uuid::try_parse(card_profile_id)
            .map_err(|e| InvalidUpdateDeckCard::CardProfileId(e))?;
        let update_quantity = UpdateQuantity::new(update_quantity)?;
        Ok(Self {
            user_id,
            deck_id,
            card_profile_id,
            update_quantity,
        })
    }
}
