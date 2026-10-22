//! Get deck profile operation (metadata only, no cards).

use uuid::Uuid;

use crate::domain::deck::DeckProfile;
use crate::domain::deck::requests::create_deck_card::CreateDeckCard;
use crate::domain::deck::requests::delete_deck_card::DeleteDeckCard;
use crate::domain::deck::requests::update_deck_card::UpdateDeckCard;
use crate::domain::deck::requests::update_deck_profile::UpdateDeckProfile;

/// Request to retrieve deck profile (metadata).
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

impl From<&DeckProfile> for GetDeckProfile {
    fn from(value: &DeckProfile) -> Self {
        Self {
            deck_id: value.id,
            user_id: value.user_id,
        }
    }
}

impl From<&UpdateDeckProfile> for GetDeckProfile {
    fn from(value: &UpdateDeckProfile) -> Self {
        Self {
            deck_id: value.deck_id,
            user_id: value.user_id,
        }
    }
}

impl From<&CreateDeckCard> for GetDeckProfile {
    fn from(value: &CreateDeckCard) -> Self {
        Self {
            deck_id: value.deck_id,
            user_id: value.user_id,
        }
    }
}

impl From<&UpdateDeckCard> for GetDeckProfile {
    fn from(value: &UpdateDeckCard) -> Self {
        Self {
            deck_id: value.deck_id,
            user_id: value.user_id,
        }
    }
}

impl From<&DeleteDeckCard> for GetDeckProfile {
    fn from(value: &DeleteDeckCard) -> Self {
        Self {
            deck_id: value.deck_id,
            user_id: value.user_id,
        }
    }
}
