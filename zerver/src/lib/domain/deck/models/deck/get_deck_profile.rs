//! Get deck profile operation (metadata only, no cards).
//!
//! Retrieves deck configuration and ownership information.
//! Used for authorization checks before deck operations.

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

/// Errors that can occur when retrieving a deck profile.
#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum GetDeckProfileError {
    /// Deck ID doesn't exist in database.
    #[error("deck profile not found")]
    NotFound,
    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),
    /// Database returned invalid data.
    #[error("deck profile found but database returned invalid object: {0}")]
    DeckProfileFromDb(anyhow::Error),
    /// Requesting user doesn't own this deck.
    #[error("deck does not belong to authenticated user")]
    Forbidden,
}

/// Request to retrieve deck profile (metadata).
///
/// # Authorization
///
/// Only the deck owner can view their deck profile.
///
/// # Example
///
/// ```rust,ignore
/// let get = GetDeckProfile::new(user_id, deck_id);
/// let profile = deck_service.get_deck_profile(&get).await?;
/// ```
#[derive(Debug, Clone)]
pub struct GetDeckProfile {
    /// Requesting user (for authorization).
    pub user_id: Uuid,
    /// Deck to retrieve.
    pub deck_id: Uuid,
}

impl GetDeckProfile {
    /// Creates a new deck profile retrieval request.
    pub fn new(user_id: Uuid, deck_id: Uuid) -> Self {
        Self { user_id, deck_id }
    }
}

#[cfg(feature = "zerver")]
impl From<&DeckProfile> for GetDeckProfile {
    fn from(value: &DeckProfile) -> Self {
        Self {
            deck_id: value.id,
            user_id: value.user_id,
        }
    }
}

#[cfg(feature = "zerver")]
impl From<&UpdateDeckProfile> for GetDeckProfile {
    fn from(value: &UpdateDeckProfile) -> Self {
        Self {
            deck_id: value.deck_id,
            user_id: value.user_id,
        }
    }
}

#[cfg(feature = "zerver")]
impl From<&CreateDeckCard> for GetDeckProfile {
    fn from(value: &CreateDeckCard) -> Self {
        Self {
            deck_id: value.deck_id,
            user_id: value.user_id,
        }
    }
}

#[cfg(feature = "zerver")]
impl From<&UpdateDeckCard> for GetDeckProfile {
    fn from(value: &UpdateDeckCard) -> Self {
        Self {
            deck_id: value.deck_id,
            user_id: value.user_id,
        }
    }
}

#[cfg(feature = "zerver")]
impl From<&DeleteDeckCard> for GetDeckProfile {
    fn from(value: &DeleteDeckCard) -> Self {
        Self {
            deck_id: value.deck_id,
            user_id: value.user_id,
        }
    }
}
