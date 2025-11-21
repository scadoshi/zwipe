use thiserror::Error;
use uuid::Uuid;

#[cfg(feature = "zerver")]
use crate::domain::deck::models::deck::get_deck_profile::GetDeckProfileError;

#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum DeleteDeckCardError {
    #[error("deck card not found")]
    NotFound,
    #[error(transparent)]
    Database(anyhow::Error),
    #[error(transparent)]
    GetDeckProfileError(GetDeckProfileError),
    #[error("deck does not belong to requesting user")]
    Forbidden,
}

#[cfg(feature = "zerver")]
impl From<GetDeckProfileError> for DeleteDeckCardError {
    fn from(value: GetDeckProfileError) -> Self {
        Self::GetDeckProfileError(value)
    }
}

#[derive(Debug, Error)]
pub enum InvalidDeleteDeckCard {
    #[error(transparent)]
    DeckId(uuid::Error),
    #[error(transparent)]
    CardProfileId(uuid::Error),
}

#[derive(Debug, Clone)]
pub struct DeleteDeckCard {
    pub user_id: Uuid,
    pub deck_id: Uuid,
    pub card_profile_id: Uuid,
}

impl DeleteDeckCard {
    pub fn new(
        user_id: Uuid,
        deck_id: &str,
        card_profile_id: &str,
    ) -> Result<Self, InvalidDeleteDeckCard> {
        let deck_id = Uuid::try_parse(deck_id).map_err(InvalidDeleteDeckCard::DeckId)?;
        let card_profile_id =
            Uuid::try_parse(card_profile_id).map_err(InvalidDeleteDeckCard::CardProfileId)?;
        Ok(Self {
            user_id,
            deck_id,
            card_profile_id,
        })
    }
}
