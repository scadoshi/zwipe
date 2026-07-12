//! Database-to-domain conversion for card profiles.

use chrono::{DateTime, Utc};
use sqlx_macros::FromRow;
use std::collections::BTreeMap;
use uuid::Uuid;
use zwipe_core::domain::card::{card_profile::CardProfile, mechanical_category::CardRole};

/// Raw database card profile record (unvalidated data from PostgreSQL).
#[derive(Debug, Clone, FromRow)]
#[allow(missing_docs)]
pub struct DatabaseCardProfile {
    pub scryfall_data_id: Uuid,
    pub is_token: bool,
    pub mechanical_categories: Option<serde_json::Value>,
    pub oracle_tags: Option<serde_json::Value>,
    pub oracle_tags_by_role: Option<serde_json::Value>,
    pub other_oracle_tags: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<DatabaseCardProfile> for CardProfile {
    fn from(value: DatabaseCardProfile) -> Self {
        let mechanical_categories: Vec<CardRole> = value
            .mechanical_categories
            .and_then(|v| serde_json::from_value::<Vec<String>>(v).ok())
            .map(|strings| {
                strings
                    .iter()
                    .filter_map(|s| CardRole::try_from(s.as_str()).ok())
                    .collect()
            })
            .unwrap_or_default();

        let oracle_tags = value
            .oracle_tags
            .and_then(|v| serde_json::from_value::<Vec<String>>(v).ok())
            .unwrap_or_default();

        let oracle_tags_by_role = value
            .oracle_tags_by_role
            .and_then(|v| serde_json::from_value::<BTreeMap<String, Vec<String>>>(v).ok())
            .unwrap_or_default();

        let other_oracle_tags = value
            .other_oracle_tags
            .and_then(|v| serde_json::from_value::<Vec<String>>(v).ok())
            .unwrap_or_default();

        Self {
            scryfall_data_id: value.scryfall_data_id,
            is_token: value.is_token,
            // Phase M dual-emit: card_roles mirrors mechanical_categories exactly.
            card_roles: mechanical_categories.clone(),
            mechanical_categories,
            oracle_tags,
            oracle_tags_by_role,
            other_oracle_tags,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
