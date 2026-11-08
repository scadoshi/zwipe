//! Deck metadata (profile without cards).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use crate::domain::deck::format::Format;
use crate::domain::deck::{DeckName, DeckOtherTag, PowerLevel};

/// Highest land target a deck may set. No format plays more than 100 cards.
pub const MAX_LAND_TARGET: i32 = 100;

/// Whether `target` is a land count a deck may set: 0 through
/// [`MAX_LAND_TARGET`].
#[must_use]
pub fn is_valid_land_target(target: i32) -> bool {
    (0..=MAX_LAND_TARGET).contains(&target)
}

/// Whether `target` is a budget a deck may set: a finite amount of zero or
/// more.
#[must_use]
pub fn is_valid_price_target(target: f64) -> bool {
    target.is_finite() && target >= 0.0
}

/// Deck metadata without card list.
#[derive(Debug, Clone, Serialize, PartialEq, Deserialize)]
pub struct DeckProfile {
    /// Unique deck identifier.
    pub id: Uuid,
    /// Validated deck name.
    pub name: DeckName,
    /// Optional commander card ID (for Commander format).
    pub commander_id: Option<Uuid>,
    /// Optional partner commander card ID (Partner / Friends Forever / Doctor's Companion).
    pub partner_commander_id: Option<Uuid>,
    /// Optional background enchantment card ID (Choose a Background).
    pub background_id: Option<Uuid>,
    /// Optional signature spell card ID (Oathbreaker).
    pub signature_spell_id: Option<Uuid>,
    /// Optional deck format.
    pub format: Option<Format>,
    /// Deck archetype/strategy tags, as server-delivered **slugs** (e.g. `aggro`).
    /// A plain `Vec<String>` (not the `DeckTag` enum) so a server-added deck tag can
    /// be selected + stored without a client release; labels via `deck_tag_label`.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Power level (official Commander Bracket). `None` = unset. `#[serde(default)]`
    /// so an older client reading a payload without it parses to `None`.
    #[serde(default)]
    pub power_level: Option<PowerLevel>,
    /// Secondary, non-gameplay labels (Budget, Jank, …). `#[serde(default)]` so
    /// older payloads without the field parse to an empty vec. Lossy-deserialized.
    #[serde(default, deserialize_with = "crate::serde_helpers::lossy_vec")]
    pub other_tags: Vec<DeckOtherTag>,
    /// Granular Oracle-tag slugs the deck declares as its strategy (from the
    /// `oracle_tags` catalog, e.g. `spot-removal`). Free strings, not a curated
    /// enum. `#[serde(default)]` so older payloads without the field parse empty.
    #[serde(default)]
    pub oracle_tags: Vec<String>,
    /// User-set land target, 0 through [`MAX_LAND_TARGET`] on new input. `None`
    /// falls back to the format-derived heuristic ([`Format::default_land_target`]).
    pub land_target: Option<i32>,
    /// User-set deck price target (budget), in `price_target_currency`, never
    /// negative on new input. `None` means no budget is set (no price alerts).
    pub price_target: Option<f64>,
    /// Currency for `price_target`. `None` falls back to USD.
    pub price_target_currency: Option<PriceCurrency>,
    /// Public share link token. `None` = private (default). `#[serde(default)]`
    /// so an older client reading a payload without it parses to `None`.
    #[serde(default)]
    pub share_token: Option<Uuid>,
    /// Owner of this deck (for authorization).
    pub user_id: Uuid,
    /// Total number of cards in the deck (sum of quantities).
    pub card_count: i64,
    /// Commander card name (if a commander is set).
    pub commander_name: Option<String>,
    /// Partner commander card name (if set).
    pub partner_commander_name: Option<String>,
    /// Background enchantment card name (if set).
    pub background_name: Option<String>,
    /// Signature spell card name (if set).
    pub signature_spell_name: Option<String>,
    /// Deck color identity (WUBRG short codes), derived at read time from the
    /// union of the command-zone cards' identities and every mainboard card's
    /// identity. For a Commander deck this equals the commander's legal identity
    /// (cards are always within it, and the command zone covers it even at zero
    /// cards); for other formats it reflects the colors the deck actually plays.
    /// Empty = colorless or an empty deck. `#[serde(default)]` so older payloads
    /// without the field parse to an empty vec.
    #[serde(default)]
    pub color_identity: Vec<String>,
    /// Art-crop URL for the commander's chosen printing, so a deck can show its
    /// command zone without fetching the cards themselves. Resolved server-side
    /// with the same front-face fallback as
    /// [`ScryfallData::art_crop_url`](crate::domain::card::models::scryfall_data::ScryfallData::art_crop_url),
    /// so double-faced commanders resolve to their front art.
    ///
    /// These four are `#[serde(default)]` like the fields above: a payload from
    /// a server that predates them parses to `None`, and a server that sends
    /// them is ignored by clients that don't know them yet.
    #[serde(default)]
    pub commander_art_url: Option<String>,
    /// Art-crop URL for the partner commander's printing (if set).
    #[serde(default)]
    pub partner_commander_art_url: Option<String>,
    /// Art-crop URL for the background enchantment's printing (if set).
    #[serde(default)]
    pub background_art_url: Option<String>,
    /// Art-crop URL for the signature spell's printing (if set).
    #[serde(default)]
    pub signature_spell_art_url: Option<String>,
    /// When the deck last changed: its profile, or a card added, edited or removed. `None` from a server that predates the field, which leaves it out.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
    /// The deck's total price in USD, counted as [`DeckMetrics::total_price_usd`](crate::domain::deck::DeckMetrics) counts it: main board copies at each printing's nonfoil price, plus command zone cards not already on the main board. `None` when no card has a USD price, outside the deck list, or from a server that predates the field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_price_usd: Option<f64>,
    /// The deck's total price in EUR, counted as [`Self::total_price_usd`] is.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_price_eur: Option<f64>,
    /// The deck's total price in MTGO Event Tickets, counted as [`Self::total_price_usd`] is.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_price_tix: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A payload without the command-zone art fields still parses, so a client
    /// reads older servers and its own cached payloads. The reverse direction
    /// (an older client reading a payload that *has* the fields) needs no test:
    /// the struct doesn't set `deny_unknown_fields`, so serde skips what it
    /// doesn't know.
    /// The deck list's sort fields: a payload from a server without them parses to `None`, a profile with them round-trips, and a profile without them leaves them out of the JSON.
    #[test]
    fn the_list_sort_fields_are_optional_both_ways() {
        let old = serde_json::json!({
            "id": Uuid::nil(),
            "name": "Old server",
            "commander_id": null,
            "partner_commander_id": null,
            "background_id": null,
            "signature_spell_id": null,
            "format": null,
            "land_target": null,
            "price_target": null,
            "price_target_currency": null,
            "share_token": null,
            "user_id": Uuid::nil(),
            "card_count": 0,
            "commander_name": null,
            "partner_commander_name": null,
            "background_name": null,
            "signature_spell_name": null,
        });
        let mut profile: DeckProfile = serde_json::from_value(old).unwrap();
        assert_eq!(profile.updated_at, None);
        assert_eq!(profile.total_price_usd, None);
        assert_eq!(profile.total_price_eur, None);
        assert_eq!(profile.total_price_tix, None);

        let json = serde_json::to_value(&profile).unwrap();
        for field in [
            "updated_at",
            "total_price_usd",
            "total_price_eur",
            "total_price_tix",
        ] {
            assert!(json.get(field).is_none(), "{field} is left out while None");
        }

        profile.updated_at = Some(DateTime::from_timestamp(1_762_560_000, 0).unwrap());
        profile.total_price_usd = Some(182.12);
        profile.total_price_eur = Some(160.5);
        profile.total_price_tix = Some(12.25);
        let json = serde_json::to_string(&profile).unwrap();
        let back: DeckProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(back, profile);
    }

    #[test]
    fn parses_payload_without_command_zone_art() {
        let json = serde_json::json!({
            "id": Uuid::nil(),
            "name": "Test deck",
            "commander_id": null,
            "partner_commander_id": null,
            "background_id": null,
            "signature_spell_id": null,
            "format": null,
            "land_target": null,
            "price_target": null,
            "price_target_currency": null,
            "share_token": null,
            "user_id": Uuid::nil(),
            "card_count": 0,
            "commander_name": null,
            "partner_commander_name": null,
            "background_name": null,
            "signature_spell_name": null,
        });
        let profile: DeckProfile = serde_json::from_value(json).unwrap();
        assert!(profile.commander_art_url.is_none());
        assert!(profile.partner_commander_art_url.is_none());
        assert!(profile.background_art_url.is_none());
        assert!(profile.signature_spell_art_url.is_none());
    }
}
