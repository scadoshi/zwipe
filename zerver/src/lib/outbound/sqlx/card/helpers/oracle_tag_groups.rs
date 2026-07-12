//! Group each card's oracle tags under the coarse role they fall beneath, for the
//! card display. Computed server-side so the role<->tag mapping and the noise
//! filter update on deploy, without waiting on mobile client releases. See
//! `context/plans/otags/`.
//!
//! Writes two `card_profiles` columns per card:
//! - `oracle_tags_by_role`: `{ role: [tags] }` for the card's tags that fall in a
//!   role's subtree (from `CATEGORY_ROOTS`) or match a `ROLE_TAG_OVERRIDES` leaf
//!   patch. Roles with neither (e.g. Tokens) never appear here and stay
//!   non-expandable on the client.
//! - `other_oracle_tags`: the card's functional tags under no role, with the
//!   structural noise stripped (mirrors `is_noise_oracle_tag` in zwipe-core).

use anyhow::Context;
use sqlx::PgPool;
use zwipe_core::domain::card::oracle_tag::NOISE_ORACLE_TAG_SLUGS;

use super::derive_categories::{CATEGORY_ROOTS, override_pairs};

/// Rebuilds `card_profiles.oracle_tags_by_role` and `.other_oracle_tags` from the
/// tag hierarchy + [`CATEGORY_ROOTS`]. Returns rows affected.
pub async fn refresh_oracle_tag_groups(pool: &PgPool) -> anyhow::Result<u64> {
    // Flatten the role map into two parallel arrays (role, root_slug) for unnest.
    let mut roles: Vec<String> = Vec::new();
    let mut root_slugs: Vec<String> = Vec::new();
    for (role, roots) in CATEGORY_ROOTS {
        for root in *roots {
            roles.push((*role).to_string());
            root_slugs.push((*root).to_string());
        }
    }
    let noise: Vec<String> = NOISE_ORACLE_TAG_SLUGS
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    let (ov_roles, ov_slugs) = override_pairs();

    // The `other` bucket's noise predicate mirrors `is_noise_oracle_tag`: the
    // explicit list is bound as $3; the four slug patterns are inlined below.
    let result = sqlx::query(
        "WITH RECURSIVE roots(role, root_slug) AS (
             SELECT r, s FROM unnest($1::text[], $2::text[]) AS t(r, s)
         ),
         subtree(role, id, slug) AS (
             SELECT r.role, ot.id, ot.slug FROM roots r JOIN oracle_tags ot ON ot.slug = r.root_slug
             UNION
             SELECT st.role, c.id, c.slug FROM subtree st JOIN oracle_tags c ON st.id = ANY(c.parent_ids)
         ),
         membership(role, slug) AS (
             SELECT DISTINCT role, slug FROM subtree
             UNION
             -- exact leaf overrides: matched as-is, no subtree expansion
             SELECT r, s FROM unnest($4::text[], $5::text[]) AS o(r, s)
         ),
         grouped(oracle_id, role, tags) AS (
             SELECT co.oracle_id, m.role, jsonb_agg(DISTINCT co.oracle_tag ORDER BY co.oracle_tag)
             FROM card_oracle_tags co JOIN membership m ON m.slug = co.oracle_tag
             GROUP BY co.oracle_id, m.role
         ),
         by_role(oracle_id, obj) AS (
             SELECT oracle_id, jsonb_object_agg(role, tags) FROM grouped GROUP BY oracle_id
         ),
         other(oracle_id, arr) AS (
             SELECT co.oracle_id, jsonb_agg(DISTINCT co.oracle_tag ORDER BY co.oracle_tag)
             FROM card_oracle_tags co
             WHERE NOT EXISTS (SELECT 1 FROM membership m WHERE m.slug = co.oracle_tag)
               AND co.oracle_tag <> ALL($3::text[])
               AND co.oracle_tag NOT LIKE '%-vanilla'
               AND co.oracle_tag NOT LIKE '%-name'
               AND co.oracle_tag <> 'alliteration'
               AND co.oracle_tag NOT LIKE 'type-addition-%'
             GROUP BY co.oracle_id
         )
         UPDATE card_profiles cp
         SET oracle_tags_by_role = COALESCE(br.obj, '{}'::jsonb),
             other_oracle_tags = COALESCE(ot.arr, '[]'::jsonb)
         FROM scryfall_data sd
         LEFT JOIN by_role br ON br.oracle_id = sd.oracle_id
         LEFT JOIN other ot ON ot.oracle_id = sd.oracle_id
         WHERE cp.scryfall_data_id = sd.id",
    )
    .bind(&roles)
    .bind(&root_slugs)
    .bind(&noise)
    .bind(&ov_roles)
    .bind(&ov_slugs)
    .execute(pool)
    .await
    .context("failed to refresh oracle_tags_by_role / other_oracle_tags")?;
    Ok(result.rows_affected())
}
