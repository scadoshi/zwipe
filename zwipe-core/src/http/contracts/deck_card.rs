//! Deck card operation HTTP request contracts.

use serde::{Deserialize, Serialize};

/// Add card to deck request body.
#[derive(Debug, Deserialize, Serialize)]
pub struct HttpCreateDeckCard {
    /// Scryfall data ID of the card to add.
    pub scryfall_data_id: String,
    /// Initial quantity.
    pub quantity: i32,
}

impl HttpCreateDeckCard {
    /// Creates a new add-card-to-deck request.
    pub fn new(scryfall_data_id: &str, quantity: i32) -> Self {
        Self {
            scryfall_data_id: scryfall_data_id.to_string(),
            quantity,
        }
    }
}

/// Card quantity update request body.
///
/// `update_quantity` is a **delta** added to the current quantity, not an absolute value.
/// For example, `1` adds one copy, `-1` removes one copy.
#[derive(Debug, Deserialize, Serialize)]
pub struct HttpUpdateDeckCard {
    pub update_quantity: i32,
}

impl HttpUpdateDeckCard {
    /// Creates a new quantity update request.
    pub fn new(update_quantity: i32) -> Self {
        Self { update_quantity }
    }
}

/// Import deck cards request body.
#[derive(Debug, Deserialize, Serialize)]
pub struct HttpImportDeckCards {
    /// Plain-text decklist (one card per line).
    pub text: String,
}
