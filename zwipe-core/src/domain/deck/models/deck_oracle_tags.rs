//! Deck-level oracle tags: the granular functional tags a deck declares as its
//! strategy. Unlike [`DeckTag`](super::deck_tag::DeckTag) / `DeckOtherTag`
//! (curated enums), these are free slug strings from the `oracle_tags` catalog
//! (e.g. `spot-removal`), so validation is just dedupe + a cap - no enum parse.
//! Picking an archetype seeds these client-side (the `DeckTag → otag-set` map
//! arrives in Phase 3 Slice B).

use std::collections::HashSet;

/// Maximum oracle tags a deck may declare. Higher than `MAX_DECK_TAGS` (5)
/// because oracle tags are granular - a deck legitimately touches many.
pub const MAX_DECK_ORACLE_TAGS: usize = 30;

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
}
