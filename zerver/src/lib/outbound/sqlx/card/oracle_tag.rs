//! Database-to-domain conversion for oracle tag catalog entries.

use sqlx_macros::FromRow;
use zwipe_core::domain::card::oracle_tag::{CURATED_ORACLE_TAGS, OracleTag};

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
        // Curation is a server decision, stamped here from the compiled
        // list, so a retune ships on a deploy. A slug that is no longer
        // served cannot be marked curated, which is what the client used to
        // filter for by hand.
        let curated = CURATED_ORACLE_TAGS.contains(&value.slug.as_str());
        Self {
            slug: value.slug,
            label: value.label,
            description: value.description,
            parent_slugs: value.parent_slugs,
            curated,
        }
    }
}
