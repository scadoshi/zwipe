//! Derive card categories from Oracle Tags (the retirement of the regex heuristic).
//!
//! For the ~18 `MechanicalCategory` variants that map cleanly to an Oracle-tag
//! subtree, plus `Tokens` from Scryfall's `all_parts` token metadata, this computes
//! `card_profiles.mechanical_categories` directly from the community gold standard.
//! See `context/plans/otags/` (Phase 2, Option A).
//!
//! NOT covered here (kept as regex heuristics — no clean otag concept): `Pump`,
//! `Stax`, `Protection`, `GraveyardHate`. `Finisher` is dropped. When this is wired
//! into `zervice`, those four are merged in from the trimmed `oracle_tag_gaps.rs`
//! heuristic; on its own this helper writes only the otag-derived + Tokens portion.

use anyhow::Context;
use sqlx::PgPool;

/// `MechanicalCategory` (serde snake_case) → the Oracle-tag root slug(s) whose
/// subtree defines it. Each root is expanded through `oracle_tags.parent_ids`.
/// The 4 stragglers + Finisher are intentionally absent (see module docs).
pub const CATEGORY_ROOTS: &[(&str, &[&str])] = &[
    (
        "anthem",
        &[
            "anthem",
            "keyword-anthem",
            "power-boost-to-all",
            "toughness-boost-to-all",
        ],
    ),
    ("blink", &["flicker"]),
    ("burn", &["burn"]),
    ("copy", &["copy", "clone"]),
    ("counterspell", &["counterspell"]),
    (
        "counters",
        &[
            "gains-pp-counters",
            "gives-pp-counters",
            "repeatable-pp-counters",
            "counters-matter",
        ],
    ),
    ("drain", &["drain-life"]),
    ("draw", &["card-advantage"]),
    ("evasion", &["evasion"]),
    ("lifegain", &["lifegain"]),
    ("mill", &["mill"]),
    ("ramp", &["ramp", "mana-producer"]),
    ("recursion", &["recursion", "reanimate"]),
    ("removal", &["removal"]),
    ("sacrifice", &["sacrifice-outlet"]),
    ("tutor", &["tutor"]),
    ("untap", &["untapper"]),
    ("wipe", &["sweeper"]),
];

/// Rebuilds `card_profiles.mechanical_categories` from Oracle Tags: expands each
/// category's roots through the tag hierarchy, unions the matching cards, adds
/// `Tokens` for any card whose `all_parts` contains a token component, and writes
/// the sorted category array per printing. Returns rows affected.
///
/// Writes the **otag-derived + Tokens portion only** — the 4 heuristic stragglers
/// are merged separately at integration (see module docs). Not yet wired into the
/// `zervice` pipeline.
pub async fn derive_categories(pool: &PgPool) -> anyhow::Result<u64> {
    // Flatten the map into two parallel arrays (category, root_slug), passed via unnest.
    let mut categories: Vec<String> = Vec::new();
    let mut root_slugs: Vec<String> = Vec::new();
    for (cat, roots) in CATEGORY_ROOTS {
        for root in *roots {
            categories.push((*cat).to_string());
            root_slugs.push((*root).to_string());
        }
    }

    let result = sqlx::query(
        "WITH RECURSIVE roots(category, root_slug) AS (
             SELECT c, r FROM unnest($1::text[], $2::text[]) AS t(c, r)
         ),
         seeded(category, id, slug) AS (
             SELECT r.category, ot.id, ot.slug FROM roots r JOIN oracle_tags ot ON ot.slug = r.root_slug
         ),
         subtree(category, id, slug) AS (
             SELECT category, id, slug FROM seeded
             UNION
             SELECT st.category, c.id, c.slug FROM subtree st JOIN oracle_tags c ON st.id = ANY(c.parent_ids)
         ),
         derived(oracle_id, category) AS (
             SELECT DISTINCT co.oracle_id, s.category FROM card_oracle_tags co JOIN subtree s ON s.slug = co.oracle_tag
             UNION
             SELECT DISTINCT sd.oracle_id, 'tokens' FROM scryfall_data sd
               WHERE sd.all_parts @> '[{\"component\":\"token\"}]'::jsonb AND sd.oracle_id IS NOT NULL
         ),
         agg(oracle_id, cats) AS (
             SELECT oracle_id, jsonb_agg(DISTINCT category ORDER BY category) AS cats FROM derived GROUP BY oracle_id
         )
         UPDATE card_profiles cp
         SET mechanical_categories = COALESCE(a.cats, '[]'::jsonb)
         FROM scryfall_data sd
         LEFT JOIN agg a ON a.oracle_id = sd.oracle_id
         WHERE cp.scryfall_data_id = sd.id",
    )
    .bind(&categories)
    .bind(&root_slugs)
    .execute(pool)
    .await
    .context("failed to derive mechanical_categories from oracle tags")?;
    Ok(result.rows_affected())
}

#[cfg(test)]
mod tests {
    use super::*;
    use zwipe_core::domain::card::mechanical_category::MechanicalCategory;

    /// Every mapped category string must round-trip to a real `MechanicalCategory`,
    /// so the map can't drift from the enum's snake_case serde.
    #[test]
    fn category_strings_are_valid_variants() {
        for (cat, _) in CATEGORY_ROOTS {
            assert!(
                MechanicalCategory::try_from(*cat).is_ok(),
                "'{cat}' is not a MechanicalCategory variant",
            );
        }
    }

    /// The map covers exactly the otag-derived set (18) — not the 4 stragglers,
    /// Tokens (all_parts), or Finisher (dropped).
    #[test]
    fn map_excludes_stragglers_tokens_and_finisher() {
        let mapped: Vec<&str> = CATEGORY_ROOTS.iter().map(|(c, _)| *c).collect();
        for excluded in [
            "pump",
            "stax",
            "protection",
            "graveyard_hate",
            "tokens",
            "finisher",
        ] {
            assert!(
                !mapped.contains(&excluded),
                "{excluded} should not be otag-mapped"
            );
        }
        assert_eq!(mapped.len(), 18);
    }
}
