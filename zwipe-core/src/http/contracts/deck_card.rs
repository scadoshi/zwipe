//! Deck card operation HTTP request contracts.

use crate::domain::card::scryfall_data::ScryfallData;
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
    /// Board to place the card on ("deck", "maybeboard", "sideboard"). Defaults to "deck" if absent.
    pub board: Option<String>,
}

impl HttpCreateDeckCard {
    /// Creates a new add-card-to-deck request from card data.
    ///
    /// Takes `&ScryfallData` to extract the correct IDs, preventing callers
    /// from accidentally mixing up `scryfall_data_id` and `oracle_id`.
    pub fn new(scryfall_data: &ScryfallData, quantity: i32, board: Option<String>) -> Self {
        Self {
            scryfall_data_id: scryfall_data.id.to_string(),
            oracle_id: scryfall_data.oracle_id.map(|id| id.to_string()).unwrap_or_default(),
            quantity,
            board,
        }
    }
}

/// Card update request body.
///
/// At least one field must be provided. `update_quantity` is a **delta** added
/// to the current quantity, not an absolute value (e.g. `1` adds one copy,
/// `-1` removes one copy). `board` sets the card's board ("deck", "maybeboard", "sideboard").
#[derive(Debug, Deserialize, Serialize)]
pub struct HttpUpdateDeckCard {
    /// Quantity delta (positive = add copies, negative = remove copies).
    pub update_quantity: Option<i32>,
    /// Move card to this board ("deck", "maybeboard", "sideboard").
    pub board: Option<String>,
    /// Change the selected printing (new Scryfall data ID).
    pub scryfall_data_id: Option<String>,
}

impl HttpUpdateDeckCard {
    /// Creates a new update request.
    pub fn new(update_quantity: Option<i32>, board: Option<String>) -> Self {
        Self {
            update_quantity,
            board,
            scryfall_data_id: None,
        }
    }

    /// Creates an update request that changes the selected printing.
    pub fn with_printing(scryfall_data_id: &str) -> Self {
        Self {
            update_quantity: None,
            board: None,
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
