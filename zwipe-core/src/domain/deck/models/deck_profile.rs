//! Deck metadata (profile without cards).

use crate::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use crate::domain::deck::{DeckName, DeckOtherTag, DeckTag, PowerLevel, format::Format};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    /// Deck archetype/strategy tags.
    pub tags: Vec<DeckTag>,
    /// Power level (WotC Commander Bracket). `None` = unset. `#[serde(default)]`
    /// so an older client reading a payload without it parses to `None`.
    #[serde(default)]
    pub power_level: Option<PowerLevel>,
    /// Secondary, non-gameplay labels (Budget, Jank, …). `#[serde(default)]` so
    /// older payloads without the field parse to an empty vec.
    #[serde(default)]
    pub other_tags: Vec<DeckOtherTag>,
    /// User-set land target. `None` falls back to the format-derived heuristic
    /// ([`Format::default_land_target`]).
    pub land_target: Option<i32>,
    /// User-set deck price target (budget), in `price_target_currency`. `None`
    /// means no budget is set (no price alerts).
    pub price_target: Option<f64>,
    /// Currency for `price_target`. `None` falls back to USD.
    pub price_target_currency: Option<PriceCurrency>,
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
}
