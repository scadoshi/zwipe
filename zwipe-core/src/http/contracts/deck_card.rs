//! Deck card operation HTTP request contracts.

use serde::{Deserialize, Serialize};

use crate::domain::card::scryfall_data::ScryfallData;
use crate::domain::deck::ImportMode;
use crate::http::helpers::Patch;

/// Add card to deck request body.
#[derive(Debug, Clone, Deserialize, Serialize)]
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

/// Card patch request body (idempotent).
///
/// At least one field must be provided. `quantity` is an **absolute** value
/// to set ("3 copies"), so replaying the request is harmless. Removing a
/// card stays an explicit DELETE; a quantity below 1 is rejected.
///
/// Every field here is non-clearable (a deck card always has all four), so
/// per the RFC 7396 resolution an explicit `null` is a 422, enforced by the
/// handler; absent means untouched. [`Patch`] fields keep null and absent
/// distinguishable on decode. The constructors never emit null.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HttpPatchDeckCard {
    /// Absolute quantity to set (≥ 1). Absent = untouched.
    #[serde(default, skip_serializing_if = "Patch::is_unchanged")]
    pub quantity: Patch<i32>,
    /// Move card to this board ("deck", "maybeboard", "sideboard").
    #[serde(default, skip_serializing_if = "Patch::is_unchanged")]
    pub board: Patch<String>,
    /// Change the selected printing (new Scryfall data ID).
    #[serde(default, skip_serializing_if = "Patch::is_unchanged")]
    pub scryfall_data_id: Patch<String>,
    /// Star (true) or unstar (false) this card as a deck MVP. Absent = untouched.
    #[serde(default, skip_serializing_if = "Patch::is_unchanged")]
    pub mvp: Patch<bool>,
}

/// Absent-or-set: the constructor inputs can't express null by design.
fn set_or_absent<T>(value: Option<T>) -> Patch<T> {
    value.map_or(Patch::Unchanged, |v| Patch::Set(Some(v)))
}

impl HttpPatchDeckCard {
    /// Creates a new patch request (quantity and/or board).
    pub fn new(quantity: Option<i32>, board: Option<String>) -> Self {
        Self {
            quantity: set_or_absent(quantity),
            board: set_or_absent(board),
            scryfall_data_id: Patch::Unchanged,
            mvp: Patch::Unchanged,
        }
    }

    /// Creates a patch request that changes the selected printing.
    pub fn with_printing(scryfall_data_id: &str) -> Self {
        Self {
            quantity: Patch::Unchanged,
            board: Patch::Unchanged,
            scryfall_data_id: Patch::Set(Some(scryfall_data_id.to_string())),
            mvp: Patch::Unchanged,
        }
    }

    /// Creates a patch request that stars (true) or unstars (false) the card
    /// as a deck MVP.
    pub fn with_mvp(mvp: bool) -> Self {
        Self {
            quantity: Patch::Unchanged,
            board: Patch::Unchanged,
            scryfall_data_id: Patch::Unchanged,
            mvp: Patch::Set(Some(mvp)),
        }
    }
}

/// Import deck cards request body.
#[derive(Debug, Deserialize, Serialize)]
pub struct HttpImportDeckCards {
    /// Plain-text decklist (one card per line).
    pub text: String,
    /// Optional board override; when set, all imported cards are placed on
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
        assert_eq!(body.quantity, Patch::Set(Some(3)));
        assert!(body.board.is_unchanged());
        assert!(body.scryfall_data_id.is_unchanged());
        assert!(body.mvp.is_unchanged());

        let wire = serde_json::to_string(&HttpPatchDeckCard::new(Some(3), None)).unwrap();
        assert_eq!(wire, r#"{"quantity":3}"#);
    }

    #[test]
    fn patch_body_distinguishes_null_from_absent() {
        // Explicit null decodes as Set(None), the handler's 422 signal,
        // while an absent field stays Unchanged. Plain Option couldn't tell
        // these apart, which is why the fields are Patch.
        let body: HttpPatchDeckCard = serde_json::from_str(r#"{"quantity":null}"#).unwrap();
        assert_eq!(body.quantity, Patch::Set(None));
        assert!(body.board.is_unchanged());
    }

    #[test]
    fn patch_constructors_touch_only_their_field() {
        let printing = HttpPatchDeckCard::with_printing("abc");
        assert_eq!(
            printing.scryfall_data_id,
            Patch::Set(Some("abc".to_string()))
        );
        assert!(
            printing.quantity.is_unchanged()
                && printing.board.is_unchanged()
                && printing.mvp.is_unchanged()
        );

        let mvp = HttpPatchDeckCard::with_mvp(true);
        assert_eq!(mvp.mvp, Patch::Set(Some(true)));
        assert!(
            mvp.quantity.is_unchanged()
                && mvp.board.is_unchanged()
                && mvp.scryfall_data_id.is_unchanged()
        );
    }
}
