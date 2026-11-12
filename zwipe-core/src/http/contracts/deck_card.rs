//! Deck card operation HTTP request contracts.

use chrono::{DateTime, Utc};
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
    /// When the card was starred as a deck MVP, sent by an undo that restores a removed row.
    /// Absent creates it unstarred. The server keeps it only on the mainboard, only when it is
    /// not in the future and only while the deck has room under the three-star cap; otherwise the
    /// card comes in unstarred.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mvp_at: Option<DateTime<Utc>>,
}

impl HttpCreateDeckCard {
    /// Creates a new add-card-to-deck request from card data.
    ///
    /// Takes `&ScryfallData` to extract the correct IDs, preventing callers
    /// from accidentally mixing up `scryfall_data_id` and `oracle_id`.
    pub fn new(scryfall_data: &ScryfallData, quantity: i32, board: Option<String>) -> Self {
        Self {
            scryfall_data_id: scryfall_data.id.to_string(),
            oracle_id: scryfall_data.oracle_id.map_or_default(|id| id.to_string()),
            quantity,
            board,
            mvp_at: None,
        }
    }

    /// The request with `mvp_at`, the star time of the row it restores.
    #[must_use]
    pub fn with_mvp_at(self, mvp_at: Option<DateTime<Utc>>) -> Self {
        Self { mvp_at, ..self }
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

/// The query string an undo's delete carries. Sent by
/// [`DeleteUndoneDeckCard`](crate::http::endpoints::deck::DeleteUndoneDeckCard).
pub const UNDO_QUERY: &str = "undo=true";

/// Remove-card query: `?undo=true` marks the delete as the undo of an add,
/// which takes the card back out without the 30-day "doesn't fit"
/// suppression a deliberate removal leaves. Absent, it is an ordinary
/// removal. Any value that isn't `true` or `false` reads as absent, so a
/// malformed flag can't fail an ordinary delete.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct HttpDeleteDeckCardQuery {
    /// Whether this delete undoes an add.
    #[serde(default, deserialize_with = "lenient_bool")]
    pub undo: Option<bool>,
}

/// `Some` for a real `true`/`false` (a boolean, or the text `"true"` or
/// `"false"` a query string carries), `None` for anything else.
fn lenient_bool<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(match value {
        serde_json::Value::Bool(b) => Some(b),
        serde_json::Value::String(text) => text.parse().ok(),
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_undo_flag_reads_true_false_or_absent() {
        let read = |json: &str| {
            serde_json::from_str::<HttpDeleteDeckCardQuery>(json)
                .unwrap()
                .undo
        };
        assert_eq!(read(r#"{"undo":"true"}"#), Some(true));
        assert_eq!(read(r#"{"undo":"false"}"#), Some(false));
        assert_eq!(read(r#"{"undo":true}"#), Some(true));
        assert_eq!(read("{}"), None);
        assert_eq!(
            read(r#"{"undo":"banana"}"#),
            None,
            "malformed reads as absent"
        );
        assert_eq!(read(r#"{"undo":"1"}"#), None);
        assert_eq!(read(r#"{"other":"x"}"#), None, "unknown keys are ignored");
        assert_eq!(UNDO_QUERY, "undo=true");
    }

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

    #[test]
    fn a_create_without_a_star_keeps_its_old_shape() {
        let old = serde_json::json!({
            "scryfall_data_id": "7673784e-db4b-43a1-8d55-1bb9fc1e284f",
            "oracle_id": "",
            "quantity": 1,
            "board": null,
        });
        let parsed: HttpCreateDeckCard = serde_json::from_value(old.clone()).unwrap();
        assert_eq!(parsed.mvp_at, None);
        assert_eq!(
            serde_json::to_value(&parsed).unwrap(),
            old,
            "an unstarred create writes no mvp_at"
        );
    }

    #[test]
    fn a_create_carries_the_star_it_restores() {
        let at: chrono::DateTime<chrono::Utc> = "2026-01-02T03:04:05Z".parse().unwrap();
        let restore = HttpCreateDeckCard {
            scryfall_data_id: "7673784e-db4b-43a1-8d55-1bb9fc1e284f".to_owned(),
            oracle_id: String::new(),
            quantity: 1,
            board: None,
            mvp_at: None,
        }
        .with_mvp_at(Some(at));
        let back: HttpCreateDeckCard =
            serde_json::from_value(serde_json::to_value(&restore).unwrap()).unwrap();
        assert_eq!(back.mvp_at, Some(at));
    }
}
