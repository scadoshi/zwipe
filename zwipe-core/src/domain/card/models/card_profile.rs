//! User-specific card metadata and computed properties.
//!
//! CardProfile stores application-computed metadata for each card:
//! - Token status (whether this is a token vs. real card)
//! - Timestamps (when card was added/updated in database)

use super::mechanical_category::CardRole;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

/// Application metadata for a card.
///
/// Stores computed properties and timestamps for each card in the database.
/// Currently minimal, but designed for future expansion with user-specific data.
#[derive(Debug, Clone, Serialize, PartialEq, Deserialize)]
pub struct CardProfile {
    /// Scryfall UUID — the primary key shared with the `scryfall_data` table.
    pub scryfall_data_id: Uuid,
    /// Whether this is a token (not a real card).
    pub is_token: bool,
    /// Mechanical categories assigned by heuristics or AI classification.
    pub mechanical_categories: Vec<CardRole>,
    /// Community Oracle Tags (granular functional tags) carried by this card.
    /// `#[serde(default)]` so older clients that omit it still deserialize.
    #[serde(default)]
    pub oracle_tags: Vec<String>,
    /// This card's functional oracle tags grouped under the coarse role they
    /// fall beneath (role slug -> its tags). Server-computed for the card
    /// display, so the role<->tag mapping updates on deploy. `#[serde(default)]`.
    #[serde(default)]
    pub oracle_tags_by_role: BTreeMap<String, Vec<String>>,
    /// Functional oracle tags that fall under no role (the "other" bucket),
    /// noise already stripped server-side. `#[serde(default)]`.
    #[serde(default)]
    pub other_oracle_tags: Vec<String>,
    /// When this profile was created in database.
    pub created_at: DateTime<Utc>,
    /// When this profile was last updated.
    pub updated_at: DateTime<Utc>,
}
