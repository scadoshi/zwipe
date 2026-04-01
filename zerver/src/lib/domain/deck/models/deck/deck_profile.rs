//! Deck metadata (profile without cards).
//!
//! Contains deck configuration and ownership information.

use crate::domain::deck::models::deck::{deck_name::DeckName, format::Format};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Deck metadata without card list.
///
/// # Fields
///
/// - **name**: Validated deck name (1-64 chars, no profanity)
/// - **commander_id**: Optional commander card for Commander format
/// - **format**: Optional deck format (Commander, Standard, Modern, etc.)
/// - **user_id**: Deck owner (for authorization)
///
/// # Relationship to Deck
///
/// - **DeckProfile**: Metadata only (this type)
/// - **Deck**: Complete view (profile + all cards)
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
