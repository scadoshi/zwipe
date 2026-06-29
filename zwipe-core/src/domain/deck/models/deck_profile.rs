//! Deck metadata (profile without cards).

use crate::domain::deck::{DeckName, DeckTag, format::Format};
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
    /// User-set land target. `None` falls back to the format-derived heuristic
    /// ([`Format::default_land_target`]).
    pub land_target: Option<i32>,
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
