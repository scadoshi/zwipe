//! Derive card categories from Oracle Tags.
//!
//! For the ~18 `CardRole` variants that map cleanly to an Oracle-tag
//! subtree, plus `Tokens` from Scryfall's `all_parts` token metadata, this computes
//! `card_profiles.card_roles` directly from the community gold standard.
//! See `context/plans/otags/`.
//!
//! The role rules themselves (`CATEGORY_ROOTS`, `ROLE_TAG_OVERRIDES`,
//! `ROLE_TAG_EXCLUSIONS`) live in zwipe-core's
//! `domain::card::card_role::role_tags`, shared with the synergy worker, and are
//! re-exported here.
//!
//! `Pump`, `Stax`, `Protection`, `GraveyardHate` also map to an otag root, but
//! only as a *supplement*: they stay heuristic-tagged too (`oracle_tag_gaps.rs`,
//! merged in `zervice`), and their union with the otag subtree is what lets those
//! roles group their tags on the card display instead of dumping them in "Other".
//! Protection/GraveyardHate have clean roots; Pump/Stax roots are partial (they
//! capture only `combat-trick` / `tax` slices), so some of their tags still fall
//! through. `Finisher` is dropped.

use anyhow::Context;
use sqlx::PgPool;
pub use zwipe_core::domain::card::card_role::role_tags::{
    CATEGORY_ROOTS, ROLE_TAG_EXCLUSIONS, ROLE_TAG_OVERRIDES, exclusion_pairs, override_pairs,
    root_pairs,
};

/// Non-fatal typo guard: log any slug the `oracle_tags` catalog doesn't carry, so
/// a bad `ROLE_TAG_OVERRIDES` / `ROLE_TAG_EXCLUSIONS` entry surfaces in the zervice
/// log instead of silently matching nothing. `source` names the const for the log.
async fn warn_unknown_slugs(pool: &PgPool, slugs: &[String], source: &str) {
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
        tracing::warn!("{source} references unknown oracle-tag slugs: {unknown:?}");
    }
}

/// Rebuilds `card_profiles.card_roles` from Oracle Tags: expands each
/// category's roots through the tag hierarchy, unions the matching cards, adds
/// `Tokens` for any card whose `all_parts` contains a token component, and writes
/// the sorted category array per printing. Returns rows affected.
///
/// Writes the **otag-derived + Tokens portion only**; the 4 heuristic stragglers
/// are also merged in from `oracle_tag_gaps.rs` at integration (see module docs).
/// Wired into `zervice` as the first step of `derive_card_categories`.
///
/// # Errors
///
/// Returns an error when the `UPDATE` fails.
pub async fn derive_categories(pool: &PgPool) -> anyhow::Result<u64> {
    let (categories, root_slugs) = root_pairs();
    let (ov_categories, ov_slugs) = override_pairs();
    let (ex_categories, ex_slugs) = exclusion_pairs();
    warn_unknown_slugs(pool, &ov_slugs, "ROLE_TAG_OVERRIDES").await;
    warn_unknown_slugs(pool, &ex_slugs, "ROLE_TAG_EXCLUSIONS").await;

    let result = sqlx::query(
        "WITH RECURSIVE roots(category, root_slug) AS (
             SELECT c, r FROM unnest($1::text[], $2::text[]) AS t(c, r)
         ),
         excl(category, slug) AS (
             SELECT c, s FROM unnest($5::text[], $6::text[]) AS e(c, s)
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
               WHERE NOT EXISTS (SELECT 1 FROM excl e WHERE e.category = s.category AND e.slug = co.oracle_tag)
             UNION
             -- exact leaf overrides: matched as-is, no subtree expansion
             SELECT DISTINCT co.oracle_id, o.category
               FROM card_oracle_tags co JOIN unnest($3::text[], $4::text[]) AS o(category, slug) ON o.slug = co.oracle_tag
               WHERE NOT EXISTS (SELECT 1 FROM excl e WHERE e.category = o.category AND e.slug = co.oracle_tag)
             UNION
             SELECT DISTINCT sd.oracle_id, 'tokens' FROM scryfall_data sd
               WHERE sd.all_parts @> '[{\"component\":\"token\"}]'::jsonb AND sd.oracle_id IS NOT NULL
         ),
         agg(oracle_id, cats) AS (
             SELECT oracle_id, jsonb_agg(DISTINCT category ORDER BY category) AS cats FROM derived GROUP BY oracle_id
         )
         UPDATE card_profiles cp
         SET card_roles = COALESCE(a.cats, '[]'::jsonb)
         FROM scryfall_data sd
         LEFT JOIN agg a ON a.oracle_id = sd.oracle_id
         WHERE cp.scryfall_data_id = sd.id",
    )
    .bind(&categories)
    .bind(&root_slugs)
    .bind(&ov_categories)
    .bind(&ov_slugs)
    .bind(&ex_categories)
    .bind(&ex_slugs)
    .execute(pool)
    .await
    .context("failed to derive card_roles from oracle tags")?;
    Ok(result.rows_affected())
}
