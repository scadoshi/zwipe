//! Deck metadata (profile without cards).

use crate::domain::deck::{DeckName, format::Format};
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
    /// Optional deck format.
    pub format: Option<Format>,
    /// Owner of this deck (for authorization).
    pub user_id: Uuid,
    /// Total number of cards in the deck (sum of quantities).
    pub card_count: i64,
    /// Commander card name (if a commander is set).
    pub commander_name: Option<String>,
}
