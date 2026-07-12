//! Database-to-domain conversion for oracle tag catalog entries.

use sqlx_macros::FromRow;
use zwipe_core::domain::card::oracle_tag::OracleTag;

/// Raw database oracle tag catalog record (from the `oracle_tags` table, with
/// `parent_ids` already resolved to parent slugs by the query).
#[derive(Debug, Clone, FromRow)]
#[allow(missing_docs)]
pub struct DatabaseOracleTag {
    pub slug: String,
    pub label: String,
    pub description: Option<String>,
    pub parent_slugs: Vec<String>,
}

impl From<DatabaseOracleTag> for OracleTag {
    fn from(value: DatabaseOracleTag) -> Self {
        Self {
            slug: value.slug,
            label: value.label,
            description: value.description,
            parent_slugs: value.parent_slugs,
        }
    }
}
