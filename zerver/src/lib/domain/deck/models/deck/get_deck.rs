use thiserror::Error;
use uuid::Uuid;

#[cfg(feature = "zerver")]
use crate::domain::{
    card::models::{card_profile::GetCardProfileError, GetCardError},
    deck::models::{
        deck::{deck_profile::DeckProfile, update_deck_profile::UpdateDeckProfile},
        deck_card::{
            create_deck_card::CreateDeckCard, delete_deck_card::DeleteDeckCard,
            get_deck_card::GetDeckCardError, update_deck_card::UpdateDeckCard,
        },
    },
};

#[derive(Debug, Error)]
pub enum InvalidGetDeck {
    #[error(transparent)]
    DeckId(uuid::Error),
}

impl From<uuid::Error> for InvalidGetDeck {
    fn from(value: uuid::Error) -> Self {
        Self::DeckId(value)
    }
}

#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum GetDeckProfileError {
    #[error("deck profile not found")]
    NotFound,
    #[error(transparent)]
    Database(anyhow::Error),
    #[error("deck profile found but database returned invalid object: {0}")]
    DeckProfileFromDb(anyhow::Error),
    #[error("deck does not belong to authenticated user")]
    Forbidden,
}

#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum GetDeckError {
    #[error(transparent)]
    GetDeckProfileError(GetDeckProfileError),
    #[error(transparent)]
    GetDeckCardError(GetDeckCardError),
    #[error(transparent)]
    GetCardProfileError(GetCardProfileError),
    #[error(transparent)]
    GetCardError(GetCardError),
}

#[cfg(feature = "zerver")]
impl From<GetDeckProfileError> for GetDeckError {
    fn from(value: GetDeckProfileError) -> Self {
        Self::GetDeckProfileError(value)
    }
}

#[cfg(feature = "zerver")]
impl From<GetDeckCardError> for GetDeckError {
    fn from(value: GetDeckCardError) -> Self {
        Self::GetDeckCardError(value)
    }
}

#[cfg(feature = "zerver")]
impl From<GetCardProfileError> for GetDeckError {
    fn from(value: GetCardProfileError) -> Self {
        Self::GetCardProfileError(value)
    }
}

#[cfg(feature = "zerver")]
impl From<GetCardError> for GetDeckError {
    fn from(value: GetCardError) -> Self {
        Self::GetCardError(value)
    }
}

#[derive(Debug, Clone)]
pub struct GetDeck {
    pub user_id: Uuid,
    pub deck_id: Uuid,
}

impl GetDeck {
    pub fn new(user_id: Uuid, deck_id: &str) -> Result<Self, InvalidGetDeck> {
        let deck_id = Uuid::try_parse(deck_id)?;

        Ok(Self { user_id, deck_id })
    }
}

#[cfg(feature = "zerver")]
impl From<&DeckProfile> for GetDeck {
    fn from(value: &DeckProfile) -> Self {
        Self {
            deck_id: value.id.clone(),
            user_id: value.user_id.clone(),
        }
    }
}

#[cfg(feature = "zerver")]
impl From<&UpdateDeckProfile> for GetDeck {
    fn from(value: &UpdateDeckProfile) -> Self {
        Self {
            deck_id: value.deck_id.clone(),
            user_id: value.user_id.clone(),
        }
    }
}

#[cfg(feature = "zerver")]
impl From<&CreateDeckCard> for GetDeck {
    fn from(value: &CreateDeckCard) -> Self {
        Self {
            deck_id: value.deck_id.clone(),
            user_id: value.user_id.clone(),
        }
    }
}

#[cfg(feature = "zerver")]
impl From<&UpdateDeckCard> for GetDeck {
    fn from(value: &UpdateDeckCard) -> Self {
        Self {
            deck_id: value.deck_id.clone(),
            user_id: value.user_id.clone(),
        }
    }
}

#[cfg(feature = "zerver")]
impl From<&DeleteDeckCard> for GetDeck {
    fn from(value: &DeleteDeckCard) -> Self {
        Self {
            deck_id: value.deck_id.clone(),
            user_id: value.user_id.clone(),
        }
    }
}
