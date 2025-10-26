#[cfg(feature = "zerver")]
use crate::domain::deck::models::{
    deck::{deck_profile::DeckProfile, update_deck_profile::UpdateDeckProfile},
    deck_card::{
        create_deck_card::CreateDeckCard, delete_deck_card::DeleteDeckCard,
        update_deck_card::UpdateDeckCard,
    },
};
#[cfg(feature = "zerver")]
use thiserror::Error;
use uuid::Uuid;

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

#[derive(Debug, Clone)]
pub struct GetDeckProfile {
    pub user_id: Uuid,
    pub deck_id: Uuid,
}

impl GetDeckProfile {
    pub fn new(user_id: Uuid, deck_id: Uuid) -> Self {
        Self { user_id, deck_id }
    }
}

#[cfg(feature = "zerver")]
impl From<&DeckProfile> for GetDeckProfile {
    fn from(value: &DeckProfile) -> Self {
        Self {
            deck_id: value.id.clone(),
            user_id: value.user_id.clone(),
        }
    }
}

#[cfg(feature = "zerver")]
impl From<&UpdateDeckProfile> for GetDeckProfile {
    fn from(value: &UpdateDeckProfile) -> Self {
        Self {
            deck_id: value.deck_id.clone(),
            user_id: value.user_id.clone(),
        }
    }
}

#[cfg(feature = "zerver")]
impl From<&CreateDeckCard> for GetDeckProfile {
    fn from(value: &CreateDeckCard) -> Self {
        Self {
            deck_id: value.deck_id.clone(),
            user_id: value.user_id.clone(),
        }
    }
}

#[cfg(feature = "zerver")]
impl From<&UpdateDeckCard> for GetDeckProfile {
    fn from(value: &UpdateDeckCard) -> Self {
        Self {
            deck_id: value.deck_id.clone(),
            user_id: value.user_id.clone(),
        }
    }
}

#[cfg(feature = "zerver")]
impl From<&DeleteDeckCard> for GetDeckProfile {
    fn from(value: &DeleteDeckCard) -> Self {
        Self {
            deck_id: value.deck_id.clone(),
            user_id: value.user_id.clone(),
        }
    }
}
