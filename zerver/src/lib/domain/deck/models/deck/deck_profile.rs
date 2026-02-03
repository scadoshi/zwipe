//! Deck metadata (profile without cards).
//!
//! Contains deck configuration and ownership information.

use crate::domain::deck::models::deck::{copy_max::CopyMax, deck_name::DeckName};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Deck metadata without card list.
///
/// # Fields
///
/// - **name**: Validated deck name (1-64 chars, no profanity)
/// - **commander_id**: Optional commander card for Commander format
/// - **copy_max**: Optional copy limit (1 = singleton, 4 = standard, defaults to 4 if None)
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
    /// Optional copy limit (1 = singleton, 4 = standard, None = use default of 4).
    pub copy_max: Option<CopyMax>,
    /// Owner of this deck (for authorization).
    pub user_id: Uuid,
}
