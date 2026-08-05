//! Deck card operation HTTP request contracts.

use crate::domain::{card::scryfall_data::ScryfallData, deck::ImportMode};
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
            oracle_id: scryfall_data
                .oracle_id
                .map(|id| id.to_string())
                .unwrap_or_default(),
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
    /// Star (true) or unstar (false) this card as a deck MVP. Absent = untouched.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mvp: Option<bool>,
}

impl HttpUpdateDeckCard {
    /// Creates a new update request.
    pub fn new(update_quantity: Option<i32>, board: Option<String>) -> Self {
        Self {
            update_quantity,
            board,
            scryfall_data_id: None,
            mvp: None,
        }
    }

    /// Creates an update request that changes the selected printing.
    pub fn with_printing(scryfall_data_id: &str) -> Self {
        Self {
            update_quantity: None,
            board: None,
            scryfall_data_id: Some(scryfall_data_id.to_string()),
            mvp: None,
        }
    }

    /// Creates an update request that stars (true) or unstars (false) the
    /// card as a deck MVP.
    pub fn with_mvp(mvp: bool) -> Self {
        Self {
            update_quantity: None,
            board: None,
            scryfall_data_id: None,
            mvp: Some(mvp),
        }
    }
}

/// Card patch request body — the idempotent successor to
/// [`HttpUpdateDeckCard`].
///
/// At least one field must be provided. Unlike the PUT body's delta,
/// `quantity` is an **absolute** value to set ("3 copies"), so replaying the
/// request is harmless. Removing a card stays an explicit DELETE; a quantity
/// below 1 is rejected. Migration: `context/plans/patch_idempotent_updates.md`.
#[derive(Debug, Deserialize, Serialize)]
pub struct HttpPatchDeckCard {
    /// Absolute quantity to set (≥ 1). Absent = untouched.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quantity: Option<i32>,
    /// Move card to this board ("deck", "maybeboard", "sideboard").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub board: Option<String>,
    /// Change the selected printing (new Scryfall data ID).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scryfall_data_id: Option<String>,
    /// Star (true) or unstar (false) this card as a deck MVP. Absent = untouched.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mvp: Option<bool>,
}

impl HttpPatchDeckCard {
    /// Creates a new patch request (quantity and/or board).
    pub fn new(quantity: Option<i32>, board: Option<String>) -> Self {
        Self {
            quantity,
            board,
            scryfall_data_id: None,
            mvp: None,
        }
    }

    /// Creates a patch request that changes the selected printing.
    pub fn with_printing(scryfall_data_id: &str) -> Self {
        Self {
            quantity: None,
            board: None,
            scryfall_data_id: Some(scryfall_data_id.to_string()),
            mvp: None,
        }
    }

    /// Creates a patch request that stars (true) or unstars (false) the card
    /// as a deck MVP.
    pub fn with_mvp(mvp: bool) -> Self {
        Self {
            quantity: None,
            board: None,
            scryfall_data_id: None,
            mvp: Some(mvp),
        }
    }
}

/// Import deck cards request body.
#[derive(Debug, Deserialize, Serialize)]
pub struct HttpImportDeckCards {
    /// Plain-text decklist (one card per line).
    pub text: String,
    /// Optional board override — when set, all imported cards are placed on
    /// this board regardless of section headers in the text.
    /// Values: `"deck"`, `"maybeboard"`, `"sideboard"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub board: Option<String>,
    /// Add on top of each board (default), or replace each board present in
    /// the import (cards on it not in the import are removed).
    /// Values: `"add"`, `"replace"`.
    #[serde(default)]
    pub mode: ImportMode,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patch_body_round_trips_absolute_quantity() {
        let body: HttpPatchDeckCard = serde_json::from_str(r#"{"quantity":3}"#).unwrap();
        assert_eq!(body.quantity, Some(3));
        assert!(body.board.is_none());
        assert!(body.scryfall_data_id.is_none());
        assert!(body.mvp.is_none());

        let wire = serde_json::to_string(&HttpPatchDeckCard::new(Some(3), None)).unwrap();
        assert_eq!(wire, r#"{"quantity":3}"#);
    }

    #[test]
    fn patch_constructors_touch_only_their_field() {
        let printing = HttpPatchDeckCard::with_printing("abc");
        assert_eq!(printing.scryfall_data_id.as_deref(), Some("abc"));
        assert!(printing.quantity.is_none() && printing.board.is_none() && printing.mvp.is_none());

        let mvp = HttpPatchDeckCard::with_mvp(true);
        assert_eq!(mvp.mvp, Some(true));
        assert!(mvp.quantity.is_none() && mvp.board.is_none() && mvp.scryfall_data_id.is_none());
    }
}
