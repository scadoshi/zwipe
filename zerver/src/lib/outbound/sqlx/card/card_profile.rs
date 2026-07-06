//! Database-to-domain conversion for card profiles.

use chrono::{DateTime, Utc};
use sqlx_macros::FromRow;
use uuid::Uuid;
use zwipe_core::domain::card::{
    card_profile::CardProfile, mechanical_category::MechanicalCategory,
};

/// Raw database card profile record (unvalidated data from PostgreSQL).
#[derive(Debug, Clone, FromRow)]
#[allow(missing_docs)]
pub struct DatabaseCardProfile {
    pub scryfall_data_id: Uuid,
    pub is_token: bool,
    pub mechanical_categories: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<DatabaseCardProfile> for CardProfile {
    fn from(value: DatabaseCardProfile) -> Self {
        let mechanical_categories = value
            .mechanical_categories
            .and_then(|v| serde_json::from_value::<Vec<String>>(v).ok())
            .map(|strings| {
                strings
                    .iter()
                    .filter_map(|s| MechanicalCategory::try_from(s.as_str()).ok())
                    .collect()
            })
            .unwrap_or_default();

        Self {
            scryfall_data_id: value.scryfall_data_id,
            is_token: value.is_token,
            mechanical_categories,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
