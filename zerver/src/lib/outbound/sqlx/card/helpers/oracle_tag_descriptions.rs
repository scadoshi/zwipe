//! Our own oracle-tag descriptions.
//!
//! Scryfall describes only ~29% of the ~4,500 oracle tags, and its
//! [`sync_oracle_tags`](super::oracle_tags::sync_oracle_tags) does a full
//! `DELETE` + re-`INSERT` of the catalog every run, so anything written into
//! `oracle_tags.description` out of band is wiped on the next sync. To make our
//! text durable, we author it here (compiled into the binary, shipped by a normal
//! deploy) and overlay it onto the catalog **inside the same sync transaction**,
//! right after the reinsert. Ours always wins: it replaces Scryfall's description
//! where we have one and fills the blanks where we don't. The end state is that
//! every tag is described by us; grow [`ORACLE_TAG_DESCRIPTIONS`] over time,
//! priority order = highest card population first.
//!
//! Because the column itself carries the merged text post-sync, every reader
//! (the `get_oracle_tags` catalog endpoint, the deck picker's definition bar, a
//! future tag dictionary) picks it up with no serve-time merge and no client
//! change. Slugs here that the fresh catalog doesn't carry are warned about in the
//! `zervice` log (a typo would otherwise match nothing silently).

/// Authored `slug -> description` map, overlaid onto `oracle_tags.description`
/// after each Scryfall sync. Descriptions are user-facing (shown in-app), so keep
/// them short, plain, and em-dash-free. Keyed by a real oracle-tag slug; the slug
/// side is warn-checked at sync time against the freshly loaded catalog.
pub const ORACLE_TAG_DESCRIPTIONS: &[(&str, &str)] = &[
    (
        "triggered-ability",
        "Cards with a triggered ability: an effect that happens on its own when a condition is met (worded when, whenever, or at).",
    ),
    (
        "attack-trigger",
        "Has an ability that triggers whenever a creature attacks.",
    ),
    (
        "removal-creature",
        "Removal aimed specifically at creatures.",
    ),
    (
        "repeatable-removal",
        "Removal you can use more than once, usually from a permanent's ability or recursion.",
    ),
    ("removal-destroy", "Removal that destroys its target."),
    ("burn-creature", "Deals direct damage to creatures."),
    ("repeatable-lifegain", "A repeatable source of life gain."),
];

/// Flatten [`ORACLE_TAG_DESCRIPTIONS`] into parallel `(slug, description)` arrays
/// for the overlay `unnest`.
pub fn description_pairs() -> (Vec<String>, Vec<String>) {
    let mut slugs = Vec::with_capacity(ORACLE_TAG_DESCRIPTIONS.len());
    let mut descriptions = Vec::with_capacity(ORACLE_TAG_DESCRIPTIONS.len());
    for (slug, description) in ORACLE_TAG_DESCRIPTIONS {
        slugs.push((*slug).to_string());
        descriptions.push((*description).to_string());
    }
    (slugs, descriptions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// A duplicate slug would make one authored description silently shadow the
    /// other in the overlay `UPDATE`; forbid it.
    #[test]
    fn slugs_are_unique() {
        let mut seen = HashSet::new();
        for (slug, _) in ORACLE_TAG_DESCRIPTIONS {
            assert!(seen.insert(*slug), "duplicate authored slug: {slug}");
        }
    }

    /// No blank descriptions (a blank would overwrite Scryfall's with nothing).
    #[test]
    fn descriptions_are_non_blank() {
        for (slug, desc) in ORACLE_TAG_DESCRIPTIONS {
            assert!(!desc.trim().is_empty(), "blank description for {slug}");
        }
    }
}
