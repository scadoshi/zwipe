//! User-specific card metadata and computed properties.
//!
//! CardProfile stores application-computed metadata for each card:
//! - Token status (whether this is a token vs. real card)
//! - Timestamps (when card was added/updated in database)

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
    /// The coarse role axis, as server-delivered **slugs** (e.g. `graveyard_hate`).
    /// A plain `Vec<String>` (not the `CardRole` enum) on purpose: a newer server's
    /// role slug renders on the card without a client release, labels resolved by
    /// prettifying the slug / the role catalog. `#[serde(default)]`.
    #[serde(default)]
    pub card_roles: Vec<String>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_card_roles_without_legacy_mechanical_categories() {
        // Phase M sunset: the wire carries `card_roles` only; the legacy
        // `mechanical_categories` field is gone.
        let ts = DateTime::<Utc>::from_timestamp(0, 0).unwrap();
        let profile = CardProfile {
            scryfall_data_id: Uuid::nil(),
            is_token: false,
            card_roles: vec!["ramp".to_string(), "removal".to_string()],
            oracle_tags: vec![],
            oracle_tags_by_role: BTreeMap::new(),
            other_oracle_tags: vec![],
            created_at: ts,
            updated_at: ts,
        };
        let json = serde_json::to_value(&profile).unwrap();
        assert_eq!(
            json.get("card_roles"),
            Some(&serde_json::json!(["ramp", "removal"]))
        );
        assert!(
            json.get("mechanical_categories").is_none(),
            "the legacy mechanical_categories field is no longer serialized"
        );
    }

    #[test]
    fn card_roles_keep_slugs_this_binary_does_not_know() {
        // card_roles is Vec<String>, so a newer server's role slug survives
        // deserialization and renders without a client release.
        let json = r#"{"scryfall_data_id":"00000000-0000-0000-0000-000000000000",
            "is_token":false,
            "card_roles":["ramp","future_role_2099","removal"],
            "created_at":"1970-01-01T00:00:00Z","updated_at":"1970-01-01T00:00:00Z"}"#;
        let p: CardProfile = serde_json::from_str(json).unwrap();
        assert_eq!(
            p.card_roles,
            vec![
                "ramp".to_string(),
                "future_role_2099".to_string(),
                "removal".to_string(),
            ]
        );
    }
}
