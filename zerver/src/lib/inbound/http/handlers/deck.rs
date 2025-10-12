pub mod create_deck_profile;
pub mod delete_deck;
pub mod get_deck;
pub mod update_deck_profile;

use crate::domain::deck::models::deck::deck_profile::DeckProfile;
use serde::Serialize;

// ============
//  http types
// ============

#[derive(Debug, Serialize, PartialEq)]
pub struct HttpDeckProfile {
    id: String,
    name: String,
    user_id: String,
}

impl HttpDeckProfile {
    pub fn new(id: &str, name: &str, user_id: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            user_id: user_id.to_string(),
        }
    }
}

impl From<DeckProfile> for HttpDeckProfile {
    fn from(value: DeckProfile) -> Self {
        Self {
            id: value.id.to_string(),
            name: value.name.to_string(),
            user_id: value.user_id.to_string(),
        }
    }
}

impl From<&DeckProfile> for HttpDeckProfile {
    fn from(value: &DeckProfile) -> Self {
        Self {
            id: value.id.to_string(),
            name: value.name.to_string(),
            user_id: value.user_id.to_string(),
        }
    }
}
