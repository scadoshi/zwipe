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

/// Curated default oracle tags surfaced up front in any otag picker (card filter
/// and deck selection): the 24 original mechanical categories mapped to their
/// best-populated real slug, plus hand-picked functional fills. The full catalog
/// (~4,500) stays reachable via search; a picker renders only the entries its
/// fetched catalog still serves. Tuned over time.
pub const CURATED_ORACLE_TAGS: &[&str] = &[
    // the original mechanical categories -> best-populated slug
    "spot-removal",
    "evasion",
    "draw-engine",
    "repeatable-creature-tokens",
    "burn-any",
    "sacrifice-outlet-creature",
    "lifegain",
    "sweeper",
    "protects-creature",
    "combat-trick",
    "ramp",
    "tutor-to-hand",
    "reanimate-creature",
    "anthem",
    "untapper-creature",
    "drain-life",
    "copy-creature",
    "mill-self",
    "hate-graveyard",
    "gives-pp-counters",
    "counterspell",
    // functional fills
    "multi-removal",
    "removal-exile",
    "removal-bounce",
    "cantrip",
    "scry",
    "discard",
    "discard-outlet",
    "opponent-loses-life",
    "mana-sink",
    "pinger",
    "gives-haste",
    "tapper-creature",
    "death-trigger",
    "modal",
    "utility-land",
    "creaturefall",
    "martyr",
    "land-ramp",
    "gives-flying",
    "gives-trample",
    "mana-dork",
    "castable-from-exile",
    "free-cast-another",
    "castable-from-graveyard",
    "damage-prevention",
    "regrowth-creature",
    "mill-opponent",
];

/// Structural / trivia oracle-tag slugs that are noise on a player-facing card
/// display: card-name quirks, vanilla flavor, raw ability templating, mana-value
/// and border/errata metadata. The full tag set still powers filtering and
/// serving; this only trims what we *show* on a card. Conservative by design -
/// better to leak a marginal tag than hide a functional one.
///
/// Public so the server's grouping pass can bind the explicit list into the
/// "other tags" filter SQL (which mirrors [`is_noise_oracle_tag`]'s patterns).
pub const NOISE_ORACLE_TAG_SLUGS: &[&str] = &[
    "activated-ability",
    "triggered-ability",
    "intervening-if-clause",
    "delayed-trigger",
    "reflexive-trigger",
    "cast-trigger-you",
    "cast-on-resolution",
    "unique-type-line",
    "cheaper-than-mv",
    "more-expensive-than-mv",
    "mana-value-matters",
    "unique-mana-cost",
    "potentially-black-border",
    "type-errata",
    "fun-ruling",
    "digital-only-mechanics",
    "noncreature-typal",
    "symmetrical",
    "hand-neutral",
    "drawback",
    "single-target-instant-sorcery",
    "group-slug",
    "self-replacement-effect",
    "namesake-spell",
];

/// True if `slug` is structural/trivia noise that should be hidden from
/// player-facing tag displays. See [`NOISE_ORACLE_TAG_SLUGS`]; also filters the
/// card-name (`*-name`) and vanilla (`*-vanilla`) trivia families and
/// `alliteration` / `type-addition-*` by pattern.
pub fn is_noise_oracle_tag(slug: &str) -> bool {
    slug.ends_with("-vanilla")
        || slug.ends_with("-name")
        || slug == "alliteration"
        || slug.starts_with("type-addition-")
        || NOISE_ORACLE_TAG_SLUGS.contains(&slug)
}

/// Turns a tag slug into a display label: `spot-removal` -> `Spot removal`.
/// Used where the catalog's own `label` isn't on hand (a served card carries
/// only slugs).
pub fn prettify_oracle_tag_slug(slug: &str) -> String {
    let spaced = slug.replace('-', " ");
    let mut chars = spaced.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noise_is_filtered() {
        for s in [
            "alliteration",
            "french-vanilla",
            "virtual-vanilla",
            "punny-name",
            "single-english-word-name",
            "activated-ability",
            "triggered-ability",
            "type-addition-human",
            "cheaper-than-mv",
        ] {
            assert!(is_noise_oracle_tag(s), "{s} should be noise");
        }
    }

    #[test]
    fn functional_tags_are_kept() {
        for s in [
            "spot-removal",
            "ramp",
            "lifegain",
            "draw-engine",
            "reanimate-creature",
            "mana-dork",
        ] {
            assert!(!is_noise_oracle_tag(s), "{s} should be kept");
        }
    }

    #[test]
    fn prettify() {
        assert_eq!(prettify_oracle_tag_slug("spot-removal"), "Spot removal");
        assert_eq!(prettify_oracle_tag_slug("ramp"), "Ramp");
        assert_eq!(prettify_oracle_tag_slug(""), "");
    }
}
