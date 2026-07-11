//! Record types for Scryfall's Oracle Tags bulk file (the Tagger project).
//!
//! The file is a JSON array of tag objects; each carries its metadata plus the
//! list of cards (`taggings`) that hold it. We invert this at ingest into a
//! normalized `card_otags` (oracle_id -> otag) correlation. See
//! `context/plans/otags/` for the wider design.

use serde::Deserialize;
use uuid::Uuid;

/// One oracle tag from the Oracle Tags bulk file.
#[derive(Debug, Clone, Deserialize)]
pub struct OracleTag {
    /// Scryfall tag id.
    pub id: Uuid,
    /// The tag identifier we correlate cards on, e.g. `removal`.
    pub slug: String,
    /// Human-readable name.
    pub label: String,
    /// Description of the tag; present for most tags, absent for some.
    #[serde(default)]
    pub description: Option<String>,
    /// Ids of parent tags in the tag hierarchy.
    #[serde(default)]
    pub parent_ids: Vec<Uuid>,
    /// Alternate slugs that resolve to this tag.
    #[serde(default)]
    pub aliases: Vec<String>,
    /// The cards carrying this tag.
    #[serde(default)]
    pub taggings: Vec<Tagging>,
}

/// A single card tagged under an [`OracleTag`].
#[derive(Debug, Clone, Deserialize)]
pub struct Tagging {
    /// Oracle id of the tagged card. Optional as a defensive guard against
    /// non-oracle taggings; such rows are skipped at ingest.
    #[serde(default)]
    pub oracle_id: Option<Uuid>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A record shaped like the real bulk file: unknown fields (`object`, `type`,
    /// `uri`, `child_ids`, `weight`) are ignored, `description` may be null, and a
    /// tagging may lack an `oracle_id`.
    #[test]
    fn parses_real_shaped_record() {
        let json = r#"{
            "object": "tag",
            "id": "00155182-3099-4742-be68-f8b4ea259d78",
            "label": "Removal",
            "slug": "removal",
            "type": "oracle",
            "uri": "https://tagger.scryfall.com/tags/card/removal",
            "description": null,
            "parent_ids": ["23fd5e7c-3ddc-49b0-818f-bd5fabb04d8f"],
            "child_ids": [],
            "aliases": ["destroy"],
            "taggings": [
                { "oracle_id": "2445e58b-87ed-4ab2-8209-a5e1f566fba7", "weight": "median" },
                { "weight": "median" }
            ]
        }"#;

        let tag: OracleTag = serde_json::from_str(json).unwrap();
        assert_eq!(tag.slug, "removal");
        assert_eq!(tag.label, "Removal");
        assert!(tag.description.is_none());
        assert_eq!(tag.parent_ids.len(), 1);
        assert_eq!(tag.aliases, vec!["destroy".to_string()]);
        let present: Vec<bool> = tag.taggings.iter().map(|t| t.oracle_id.is_some()).collect();
        assert_eq!(present, vec![true, false]);
    }

    /// Missing optional collections default rather than failing.
    #[test]
    fn missing_optional_fields_default() {
        let json =
            r#"{ "id": "00155182-3099-4742-be68-f8b4ea259d78", "slug": "ramp", "label": "Ramp" }"#;
        let tag: OracleTag = serde_json::from_str(json).unwrap();
        assert!(tag.description.is_none());
        assert!(tag.parent_ids.is_empty());
        assert!(tag.aliases.is_empty());
        assert!(tag.taggings.is_empty());
    }
}
