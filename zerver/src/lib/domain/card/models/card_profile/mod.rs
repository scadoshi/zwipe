//! User-specific card metadata and computed properties.
//!
//! CardProfile stores application-computed metadata for each card:
//! - Commander legality (derived from Scryfall data)
//! - Token status (whether this is a token vs. real card)
//! - Timestamps (when card was added/updated in database)
//!
//! Future expansion may include user-specific data like favorites, custom notes, etc.

/// Get card profile operation.
pub mod get_card_profile;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Application metadata for a card.
///
/// Stores computed properties and timestamps for each card in the database.
/// Currently minimal, but designed for future expansion with user-specific data.
///
/// # Current Fields
///
/// - **Commander Legality**: Pre-computed from Scryfall type_line (is it a legendary creature?)
/// - **Token Status**: Whether this is a token (not a real card for deck building)
/// - **Timestamps**: Database record creation/update times
///
/// # Future Expansion
///
/// This structure is designed to support user-specific card metadata:
/// - Favorites/bookmarks
/// - Custom notes
/// - Personal ratings
/// - Usage statistics
#[derive(Debug, Clone, Serialize, PartialEq, Deserialize)]
pub struct CardProfile {
    /// Unique card profile ID.
    pub id: Uuid,
    /// Links to Scryfall card data.
    pub scryfall_data_id: Uuid,
    /// Pre-computed commander legality (legendary creature check).
    pub is_valid_commander: bool,
    /// Whether this is a token (not a real card).
    pub is_token: bool,
    /// When this profile was created in database.
    pub created_at: NaiveDateTime,
    /// When this profile was last updated.
    pub updated_at: NaiveDateTime,
}
