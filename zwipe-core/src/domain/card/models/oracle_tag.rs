//! Oracle tag catalog entry served to clients.
//!
//! Scryfall's community-maintained Oracle Tags (the Tagger project) are ingested
//! into the `oracle_tags` catalog table (see `context/plans/otags/`). This type is
//! the read-side projection the server serves and the client consumes to build the
//! otag filter picker: the slug players filter on plus the human label, definition,
//! and parent slugs for grouping. It is deliberately lighter than the ingest record
//! (no ids, taggings, or aliases) - only what a picker needs.

use serde::{Deserialize, Serialize};

/// One entry in the oracle tag catalog.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OracleTag {
    /// The tag identifier cards are correlated on and filters key on, e.g. `spot-removal`.
    pub slug: String,
    /// Human-readable name, e.g. `Spot Removal`.
    pub label: String,
    /// Plain-language definition of the tag; absent for some tags.
    pub description: Option<String>,
    /// Slugs of this tag's parents in the tag hierarchy (for grouping/curation).
    pub parent_slugs: Vec<String>,
}
