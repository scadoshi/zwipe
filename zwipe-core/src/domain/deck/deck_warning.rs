//! Deck validation warning value object.

use serde::{Deserialize, Serialize};
use std::ops::Deref;
use uuid::Uuid;

/// A deck-building warning message (informational, not blocking).
///
/// Card-specific warnings carry a `scryfall_data_id` so the client
/// can offer a "remove" action for the offending card.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeckWarning {
    message: String,
    scryfall_data_id: Option<Uuid>,
}

impl DeckWarning {
    /// Creates a deck-level warning (no specific card).
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            scryfall_data_id: None,
        }
    }

    /// Creates a card-specific warning with the offending card's ID.
    pub fn with_card(message: impl Into<String>, scryfall_data_id: Uuid) -> Self {
        Self {
            message: message.into(),
            scryfall_data_id: Some(scryfall_data_id),
        }
    }

    /// Returns the scryfall_data_id of the offending card, if any.
    pub fn scryfall_data_id(&self) -> Option<Uuid> {
        self.scryfall_data_id
    }
}

impl Deref for DeckWarning {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.message
    }
}

impl std::fmt::Display for DeckWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
