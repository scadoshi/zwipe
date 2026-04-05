//! Deck card operation HTTP request contracts.

use serde::{Deserialize, Serialize};

/// Add card to deck request body.
#[derive(Debug, Deserialize, Serialize)]
pub struct HttpCreateDeckCard {
    /// Scryfall data ID of the card to add (selected printing).
    pub scryfall_data_id: String,
    /// Oracle ID of the card (logical identity across printings).
    pub oracle_id: String,
    /// Initial quantity.
    pub quantity: i32,
    /// Whether the card is on the maybeboard. Defaults to `false` if absent.
    pub maybeboard: Option<bool>,
}

impl HttpCreateDeckCard {
    /// Creates a new add-card-to-deck request.
    pub fn new(scryfall_data_id: &str, oracle_id: &str, quantity: i32, maybeboard: Option<bool>) -> Self {
        Self {
            scryfall_data_id: scryfall_data_id.to_string(),
            oracle_id: oracle_id.to_string(),
            quantity,
            maybeboard,
        }
    }
}

/// Card update request body.
///
/// At least one field must be provided. `update_quantity` is a **delta** added
/// to the current quantity, not an absolute value (e.g. `1` adds one copy,
/// `-1` removes one copy). `maybeboard` sets the card's maybeboard status.
#[derive(Debug, Deserialize, Serialize)]
pub struct HttpUpdateDeckCard {
    /// Quantity delta (positive = add copies, negative = remove copies).
    pub update_quantity: Option<i32>,
    /// Set maybeboard status.
    pub maybeboard: Option<bool>,
    /// Change the selected printing (new Scryfall data ID).
    pub scryfall_data_id: Option<String>,
}

impl HttpUpdateDeckCard {
    /// Creates a new update request.
    pub fn new(update_quantity: Option<i32>, maybeboard: Option<bool>) -> Self {
        Self {
            update_quantity,
            maybeboard,
            scryfall_data_id: None,
        }
    }

    /// Creates an update request that changes the selected printing.
    pub fn with_printing(scryfall_data_id: &str) -> Self {
        Self {
            update_quantity: None,
            maybeboard: None,
            scryfall_data_id: Some(scryfall_data_id.to_string()),
        }
    }
}

/// Import deck cards request body.
#[derive(Debug, Deserialize, Serialize)]
pub struct HttpImportDeckCards {
    /// Plain-text decklist (one card per line).
    pub text: String,
}
