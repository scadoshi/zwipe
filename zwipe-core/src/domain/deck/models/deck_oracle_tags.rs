//! Deck-level oracle tags: the granular functional tags a deck declares as its
//! strategy. Unlike [`DeckTag`](super::deck_tag::DeckTag) / `DeckOtherTag`
//! (curated enums), these are free slug strings from the `oracle_tags` catalog
//! (e.g. `spot-removal`), so validation is just dedupe + a cap - no enum parse.
//! Picking an archetype seeds these client-side (the `DeckTag → otag-set` map
//! arrives in Phase 3 Slice B).

use super::deck_tag::DeckTag;
use std::collections::HashSet;

/// Maximum oracle tags a deck may declare. Higher than `MAX_DECK_TAGS` (5)
/// because oracle tags are granular - a deck legitimately touches many.
pub const MAX_DECK_ORACLE_TAGS: usize = 30;

/// The oracle-tag slugs seeded by a set of selected deck tags: the union of each
/// archetype's [`DeckTag::oracle_tag_slugs`], deduped, first-seen order. This is
/// what a client unions into the deck's oracle tags when archetypes are picked
/// (the flat "select a deck tag, its oracle tags pre-select" behavior).
pub fn seed_oracle_tags(tags: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    tags.iter()
        .filter_map(|slug| DeckTag::try_from(slug.as_str()).ok())
        .flat_map(|t| t.oracle_tag_slugs())
        .filter(|s| seen.insert(**s))
        .map(|s| (*s).to_string())
        .collect()
}

/// Dedupes raw oracle-tag slugs, preserving first-seen order (mirrors
/// `parse_tags`' dedupe, minus the enum validation - slugs are free strings).
pub fn dedupe_oracle_tags(raw: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    raw.iter()
        .filter(|s| seen.insert((*s).clone()))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dedupes_preserving_order() {
        let out = dedupe_oracle_tags(&[
            "spot-removal".to_string(),
            "ramp".to_string(),
            "spot-removal".to_string(),
        ]);
        assert_eq!(out, vec!["spot-removal".to_string(), "ramp".to_string()]);
    }

    #[test]
    fn seeds_and_unions_deck_tags() {
        // Aristocrats + Sacrifice overlap on sacrifice-outlet-creature + death-trigger;
        // the union dedupes them.
        let out = seed_oracle_tags(&["aristocrats".to_string(), "sacrifice".to_string()]);
        assert!(out.contains(&"sacrifice-outlet-creature".to_string()));
        assert!(out.contains(&"blood-artist-ability".to_string())); // Aristocrats-only
        assert!(out.contains(&"repeatable-sacrifice-outlet".to_string())); // Sacrifice-only
        // deduped: sacrifice-outlet-creature appears once
        assert_eq!(
            out.iter()
                .filter(|s| *s == "sacrifice-outlet-creature")
                .count(),
            1
        );
    }

    /// An unmapped archetype seeds nothing.
    #[test]
    fn unmapped_archetype_seeds_empty() {
        assert!(DeckTag::Chaos.oracle_tag_slugs().is_empty());
        assert!(seed_oracle_tags(&["chaos".to_string()]).is_empty());
    }
}
