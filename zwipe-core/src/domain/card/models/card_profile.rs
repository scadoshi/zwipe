//! User-specific card metadata and computed properties.
//!
//! CardProfile stores application-computed metadata for each card:
//! - Token status (whether this is a token vs. real card)
//! - Timestamps (when card was added/updated in database)

use super::card_role::CardRole;
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
    ///
    /// Legacy name for the coarse role axis; dual-emitted alongside `card_roles`
    /// (identical values) during the Phase M rename, and dropped at the sunset.
    /// Deserialized lossily (unknown role slugs dropped) so a newer server's roles
    /// never crash an older client — see `serde_helpers::lossy_vec`.
    #[serde(default, deserialize_with = "crate::serde_helpers::lossy_vec")]
    pub mechanical_categories: Vec<CardRole>,
    /// Canonical `card_roles` name for the coarse role axis (Phase M). Always
    /// equal to `mechanical_categories` — the server emits both so a client can
    /// read either. Lossy-deserialized (see `mechanical_categories`).
    #[serde(default, deserialize_with = "crate::serde_helpers::lossy_vec")]
    pub card_roles: Vec<CardRole>,
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
    fn card_roles_dual_emits_alongside_mechanical_categories() {
        // Phase M: the response carries both keys with identical values, so a
        // client may read either. Sunset later drops `mechanical_categories`.
        let ts = DateTime::<Utc>::from_timestamp(0, 0).unwrap();
        let profile = CardProfile {
            scryfall_data_id: Uuid::nil(),
            is_token: false,
            mechanical_categories: vec![CardRole::Ramp, CardRole::Removal],
            card_roles: vec![CardRole::Ramp, CardRole::Removal],
            oracle_tags: vec![],
            oracle_tags_by_role: BTreeMap::new(),
            other_oracle_tags: vec![],
            created_at: ts,
            updated_at: ts,
        };
        let json = serde_json::to_value(&profile).unwrap();
        let expected = serde_json::json!(["ramp", "removal"]);
        assert_eq!(json.get("mechanical_categories"), Some(&expected));
        assert_eq!(json.get("card_roles"), Some(&expected));

        // Round-trips, and a payload omitting card_roles still deserializes.
        let back: CardProfile = serde_json::from_value(json).unwrap();
        assert_eq!(back.card_roles, back.mechanical_categories);
        let legacy = r#"{"scryfall_data_id":"00000000-0000-0000-0000-000000000000",
            "is_token":false,"mechanical_categories":["ramp"],
            "created_at":"1970-01-01T00:00:00Z","updated_at":"1970-01-01T00:00:00Z"}"#;
        let old: CardProfile = serde_json::from_str(legacy).unwrap();
        assert!(
            old.card_roles.is_empty(),
            "card_roles defaults when omitted"
        );
    }

    #[test]
    fn unknown_role_slug_is_dropped_not_errored() {
        // Part 0 forward-compat: a newer server sends a role slug this binary's
        // CardRole enum doesn't know. The whole card must still deserialize, with
        // the unknown dropped and the known roles kept — not error the payload.
        let json = r#"{"scryfall_data_id":"00000000-0000-0000-0000-000000000000",
            "is_token":false,
            "mechanical_categories":["ramp","future_role_2099","removal"],
            "card_roles":["ramp","future_role_2099","removal"],
            "created_at":"1970-01-01T00:00:00Z","updated_at":"1970-01-01T00:00:00Z"}"#;
        let p: CardProfile = serde_json::from_str(json).unwrap();
        assert_eq!(
            p.mechanical_categories,
            vec![CardRole::Ramp, CardRole::Removal]
        );
        assert_eq!(p.card_roles, vec![CardRole::Ramp, CardRole::Removal]);
    }
}
