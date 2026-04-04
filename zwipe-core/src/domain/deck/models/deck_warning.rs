//! Deck validation warning value object.

use serde::{Deserialize, Serialize};
use std::ops::Deref;
use uuid::Uuid;

/// Suggested fix action for a deck warning.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WarningAction {
    /// Remove the card from the deck entirely.
    Remove,
    /// Set the card's quantity to this value.
    FixQuantity(i32),
    /// Clear the commander from the deck profile.
    ClearCommander,
}

/// A deck-building warning message (informational, not blocking).
///
/// Card-specific warnings carry a `scryfall_data_id` so the client
/// can offer a "remove" action for the offending card.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeckWarning {
    message: String,
    scryfall_data_id: Option<Uuid>,
    #[serde(default)]
    action: Option<WarningAction>,
}

impl DeckWarning {
    /// Creates a deck-level warning (no specific card).
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            scryfall_data_id: None,
            action: None,
        }
    }

    /// Creates a card-specific warning with the offending card's ID.
    pub fn with_card(message: impl Into<String>, scryfall_data_id: Uuid) -> Self {
        Self {
            message: message.into(),
            scryfall_data_id: Some(scryfall_data_id),
            action: None,
        }
    }

    /// Creates a card-specific warning with a suggested action.
    pub fn with_action(
        message: impl Into<String>,
        scryfall_data_id: Uuid,
        action: WarningAction,
    ) -> Self {
        Self {
            message: message.into(),
            scryfall_data_id: Some(scryfall_data_id),
            action: Some(action),
        }
    }

    /// Returns the scryfall_data_id of the offending card, if any.
    pub fn scryfall_data_id(&self) -> Option<Uuid> {
        self.scryfall_data_id
    }

    /// Returns the suggested action, if any.
    pub fn action(&self) -> Option<&WarningAction> {
        self.action.as_ref()
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
