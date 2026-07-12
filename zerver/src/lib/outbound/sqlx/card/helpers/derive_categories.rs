//! Derive card categories from Oracle Tags (the retirement of the regex heuristic).
//!
//! For the ~18 `MechanicalCategory` variants that map cleanly to an Oracle-tag
//! subtree, plus `Tokens` from Scryfall's `all_parts` token metadata, this computes
//! `card_profiles.mechanical_categories` directly from the community gold standard.
//! See `context/plans/otags/` (Phase 2, Option A).
//!
//! `Pump`, `Stax`, `Protection`, `GraveyardHate` also map to an otag root here, but
//! only as a *supplement*: they stay heuristic-tagged too (`oracle_tag_gaps.rs`,
//! merged in `zervice`), and their union with the otag subtree is what lets those
//! roles group their tags on the card display instead of dumping them in "Other".
//! Protection/GraveyardHate have clean roots; Pump/Stax roots are partial (they
//! capture only `combat-trick` / `tax` slices), so some of their tags still fall
//! through. `Finisher` is dropped.

use anyhow::Context;
use sqlx::PgPool;

/// `MechanicalCategory` (serde snake_case) → the Oracle-tag root slug(s) whose
/// subtree defines it. Each root is expanded through `oracle_tags.parent_ids`.
/// The 4 heuristic stragglers are also here as supplements (see module docs);
/// only `Finisher` is absent.
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
    ("graveyard_hate", &["hate-graveyard"]),
    ("lifegain", &["lifegain"]),
    ("mill", &["mill"]),
    ("protection", &["protection"]),
    ("pump", &["combat-trick"]),
    ("ramp", &["ramp", "mana-producer"]),
    ("recursion", &["recursion", "reanimate"]),
    ("removal", &["removal"]),
    ("sacrifice", &["sacrifice-outlet"]),
    ("stax", &["tax"]),
    ("tutor", &["tutor"]),
    ("untap", &["untapper"]),
    ("wipe", &["sweeper"]),
];

/// Manual one-off patches: exact oracle-tag slugs whose wording dodges Scryfall's
/// hierarchy but clearly belong to a role. Unlike [`CATEGORY_ROOTS`], these are
/// matched **exactly** — never subtree-expanded — so an entry adds precisely that
/// tag and nothing beneath it. Key = a real `MechanicalCategory` (snake_case),
/// checked by unit test; value = real oracle-tag slugs, checked by a non-fatal
/// `zervice` warn. Fed into both derivation and grouping. Grow as you audit;
/// applies on the next `zervice` run.
pub const ROLE_TAG_OVERRIDES: &[(&str, &[&str])] = &[(
    "protection",
    &["fog-selective", "damage-prevention", "phasing"],
)];

/// Flatten [`ROLE_TAG_OVERRIDES`] into parallel `(role, slug)` arrays for `unnest`.
pub fn override_pairs() -> (Vec<String>, Vec<String>) {
    let mut roles = Vec::new();
    let mut slugs = Vec::new();
    for (role, tags) in ROLE_TAG_OVERRIDES {
        for tag in *tags {
            roles.push((*role).to_string());
            slugs.push((*tag).to_string());
        }
    }
    (roles, slugs)
}

/// Non-fatal typo guard: log any override slug the `oracle_tags` catalog doesn't
/// carry, so a bad `ROLE_TAG_OVERRIDES` entry surfaces in the zervice log instead
/// of silently matching nothing.
async fn warn_unknown_override_slugs(pool: &PgPool, slugs: &[String]) {
    if slugs.is_empty() {
        return;
    }
    let unknown: Vec<String> = sqlx::query_scalar(
        "SELECT s FROM unnest($1::text[]) AS s
         WHERE NOT EXISTS (SELECT 1 FROM oracle_tags WHERE slug = s)",
    )
    .bind(slugs)
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    if !unknown.is_empty() {
        tracing::warn!("ROLE_TAG_OVERRIDES references unknown oracle-tag slugs: {unknown:?}");
    }
}

/// Rebuilds `card_profiles.mechanical_categories` from Oracle Tags: expands each
/// category's roots through the tag hierarchy, unions the matching cards, adds
/// `Tokens` for any card whose `all_parts` contains a token component, and writes
/// the sorted category array per printing. Returns rows affected.
///
/// Writes the **otag-derived + Tokens portion only** — the 4 heuristic stragglers
/// are also merged in from `oracle_tag_gaps.rs` at integration (see module docs).
/// Wired into `zervice` as the first step of `derive_card_categories`.
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
    let (ov_categories, ov_slugs) = override_pairs();
    warn_unknown_override_slugs(pool, &ov_slugs).await;

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
             -- exact leaf overrides: matched as-is, no subtree expansion
             SELECT DISTINCT co.oracle_id, o.category
               FROM card_oracle_tags co JOIN unnest($3::text[], $4::text[]) AS o(category, slug) ON o.slug = co.oracle_tag
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
    .bind(&ov_categories)
    .bind(&ov_slugs)
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

    /// Every override key must be a real card role too (the slug side is checked
    /// at runtime against the live catalog by `warn_unknown_override_slugs`).
    #[test]
    fn override_roles_are_valid_variants() {
        for (role, _) in ROLE_TAG_OVERRIDES {
            assert!(
                MechanicalCategory::try_from(*role).is_ok(),
                "'{role}' is not a MechanicalCategory variant",
            );
        }
    }

    /// The map covers the 18 otag-derived roles plus the 4 heuristic stragglers as
    /// supplements (Protection/Pump/Stax/GraveyardHate — for grouping + union
    /// coverage). Only Tokens (all_parts) and Finisher (dropped) stay out.
    #[test]
    fn map_covers_otag_roles_plus_stragglers() {
        let mapped: Vec<&str> = CATEGORY_ROOTS.iter().map(|(c, _)| *c).collect();
        for straggler in ["pump", "stax", "protection", "graveyard_hate"] {
            assert!(
                mapped.contains(&straggler),
                "{straggler} should be otag-mapped as a supplement"
            );
        }
        for excluded in ["tokens", "finisher"] {
            assert!(
                !mapped.contains(&excluded),
                "{excluded} should not be otag-mapped"
            );
        }
        assert_eq!(mapped.len(), 22);
    }
}
