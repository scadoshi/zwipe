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

/// Ranked, case-insensitive substring search over the tag catalog, shared by
/// every otag search surface (deck selector, card filter, dictionary): exact
/// slug/label matches first, then slug/label substring matches, then
/// description-only matches, each tier alphabetical by slug. A blank query
/// returns nothing — pickers show their curated/browse views instead. Callers
/// apply their own result cap.
pub fn search_oracle_tags(tags: &[OracleTag], query: &str) -> Vec<OracleTag> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return Vec::new();
    }
    let rank = |t: &OracleTag| -> Option<u8> {
        let label = t.label.to_lowercase();
        if t.slug == q || label == q {
            Some(0)
        } else if t.slug.contains(&q) || label.contains(&q) {
            Some(1)
        } else if t
            .description
            .as_deref()
            .is_some_and(|d| d.to_lowercase().contains(&q))
        {
            Some(2)
        } else {
            None
        }
    };
    let mut ranked: Vec<(u8, OracleTag)> = tags
        .iter()
        .filter_map(|t| rank(t).map(|r| (r, t.clone())))
        .collect();
    ranked.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.slug.cmp(&b.1.slug)));
    ranked.into_iter().map(|(_, t)| t).collect()
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests {
    use super::*;

    fn tag(slug: &str, label: &str, description: Option<&str>) -> OracleTag {
        OracleTag {
            slug: slug.to_string(),
            label: label.to_string(),
            description: description.map(str::to_string),
            parent_slugs: Vec::new(),
        }
    }

    #[test]
    fn search_ranks_exact_then_substring_then_description() {
        let tags = vec![
            tag(
                "board-wipe",
                "Board Wipe",
                Some("Mass removal of permanents."),
            ),
            tag("removal", "Removal", Some("Gets rid of a permanent.")),
            tag(
                "spot-removal",
                "Spot Removal",
                Some("Removes a single target."),
            ),
            tag(
                "counterspell",
                "Counterspell",
                Some("A form of removal on the stack."),
            ),
        ];
        let results = search_oracle_tags(&tags, "removal");
        let slugs: Vec<&str> = results.iter().map(|t| t.slug.as_str()).collect();
        // Exact first, substring matches alphabetical next, description-only last.
        assert_eq!(
            slugs,
            vec!["removal", "spot-removal", "board-wipe", "counterspell"]
        );
    }

    #[test]
    fn search_is_case_insensitive_and_trims() {
        let tags = vec![tag("lifegain", "Lifegain", None)];
        let results = search_oracle_tags(&tags, "  LifeGain ");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].slug, "lifegain");
    }

    #[test]
    fn search_matches_label_exactly() {
        let tags = vec![
            tag("mana-dork", "Mana Dork", None),
            tag("mana-dorks-matter", "Mana Dorks Matter", None),
        ];
        let results = search_oracle_tags(&tags, "mana dork");
        assert_eq!(results[0].slug, "mana-dork");
    }

    #[test]
    fn search_blank_query_returns_nothing() {
        let tags = vec![tag("ramp", "Ramp", None)];
        assert!(search_oracle_tags(&tags, "").is_empty());
        assert!(search_oracle_tags(&tags, "   ").is_empty());
    }

    #[test]
    fn search_skips_nonmatches() {
        let tags = vec![tag("ramp", "Ramp", Some("Accelerates your mana."))];
        assert!(search_oracle_tags(&tags, "graveyard").is_empty());
    }

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
}
