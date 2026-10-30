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
//!
//! The `(role, slug)` membership itself is shared through `role_membership_ctes`
//! and `RoleMembershipBinds`, so the commander suggestions group a shared tag
//! under the same role the card display does.

use anyhow::Context;
use sqlx::PgPool;
use zwipe_core::domain::card::card_role::role_tags::{exclusion_pairs, override_pairs, root_pairs};
use zwipe_core::domain::card::oracle_tag::NOISE_ORACLE_TAG_SLUGS;

/// The six parallel arrays [`role_membership_ctes`] reads, in bind order, built
/// from zwipe-core's role rules (`domain::card::card_role::role_tags`).
/// `RoleMembership::from_hierarchy` there computes the same membership in Rust.
#[derive(Debug, Clone)]
pub struct RoleMembershipBinds {
    /// Role per root slug.
    pub roles: Vec<String>,
    /// Root slug per role.
    pub root_slugs: Vec<String>,
    /// Role per exact override slug.
    pub override_roles: Vec<String>,
    /// Exact override slug per role.
    pub override_slugs: Vec<String>,
    /// Role per exact exclusion slug.
    pub exclusion_roles: Vec<String>,
    /// Exact exclusion slug per role.
    pub exclusion_slugs: Vec<String>,
}

impl RoleMembershipBinds {
    /// Flattens the role tables for `unnest`.
    pub fn new() -> Self {
        let (roles, root_slugs) = root_pairs();
        let (override_roles, override_slugs) = override_pairs();
        let (exclusion_roles, exclusion_slugs) = exclusion_pairs();
        Self {
            roles,
            root_slugs,
            override_roles,
            override_slugs,
            exclusion_roles,
            exclusion_slugs,
        }
    }

    /// Binds the six arrays to `query`, in the order [`role_membership_ctes`]
    /// numbers them.
    pub fn bind<'q>(
        &'q self,
        query: sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments>,
    ) -> sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments> {
        query
            .bind(&self.roles)
            .bind(&self.root_slugs)
            .bind(&self.override_roles)
            .bind(&self.override_slugs)
            .bind(&self.exclusion_roles)
            .bind(&self.exclusion_slugs)
    }
}

impl Default for RoleMembershipBinds {
    fn default() -> Self {
        Self::new()
    }
}

/// The CTEs that end in `membership(role, slug)`: every oracle tag slug and the
/// card role it falls under, one row per pair. A tag under several roots appears
/// once per role. Goes right after `WITH RECURSIVE`; the six text arrays of a
/// [`RoleMembershipBinds`] are `$first` through `$first + 5`.
pub fn role_membership_ctes(first: usize) -> String {
    let (roots_r, roots_s, ov_r, ov_s, ex_r, ex_s) =
        (first, first + 1, first + 2, first + 3, first + 4, first + 5);
    format!(
        "roots(role, root_slug) AS (
             SELECT r, s FROM unnest(${roots_r}::text[], ${roots_s}::text[]) AS t(r, s)
         ),
         excl(role, slug) AS (
             SELECT r, s FROM unnest(${ex_r}::text[], ${ex_s}::text[]) AS e(r, s)
         ),
         subtree(role, id, slug) AS (
             SELECT r.role, ot.id, ot.slug FROM roots r JOIN oracle_tags ot ON ot.slug = r.root_slug
             UNION
             -- unnested parents hash-join; `= ANY(parent_ids)` forces a nested loop
             SELECT st.role, c.id, c.slug
             FROM subtree st
             JOIN (SELECT id, slug, unnest(parent_ids) AS parent FROM oracle_tags) c
               ON c.parent = st.id
         ),
         membership(role, slug) AS (
             SELECT role, slug FROM (
                 SELECT DISTINCT role, slug FROM subtree
                 UNION
                 -- exact leaf overrides: matched as-is, no subtree expansion
                 SELECT r, s FROM unnest(${ov_r}::text[], ${ov_s}::text[]) AS o(r, s)
             ) m
             -- subtract exact (role, slug) exclusions dragged in by the hierarchy
             WHERE NOT EXISTS (SELECT 1 FROM excl e WHERE e.role = m.role AND e.slug = m.slug)
         )"
    )
}

/// Rebuilds `card_profiles.oracle_tags_by_role` and `.other_oracle_tags` from the
/// tag hierarchy + `CATEGORY_ROOTS`. Returns rows affected.
///
/// # Errors
///
/// Returns an error if the update statement fails.
pub async fn refresh_oracle_tag_groups(pool: &PgPool) -> anyhow::Result<u64> {
    let binds = RoleMembershipBinds::new();
    let noise: Vec<String> = NOISE_ORACLE_TAG_SLUGS
        .iter()
        .map(|s| (*s).to_string())
        .collect();

    // The `other` bucket's noise predicate mirrors `is_noise_oracle_tag`: the
    // explicit list is bound as $7; the four slug patterns are inlined below.
    // The SQL text is literals plus parameter numbers; the data is all bound.
    let sql = format!(
        "WITH RECURSIVE {},
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
               AND co.oracle_tag <> ALL($7::text[])
               AND co.oracle_tag NOT LIKE '%-vanilla'
               AND co.oracle_tag NOT LIKE '%-name'
               AND co.oracle_tag <> 'alliteration'
               AND co.oracle_tag NOT LIKE 'type-addition-%'
             GROUP BY co.oracle_id
         )
         UPDATE card_profiles cp
         SET oracle_tags_by_role = COALESCE(br.obj, '{{}}'::jsonb),
             other_oracle_tags = COALESCE(ot.arr, '[]'::jsonb)
         FROM scryfall_data sd
         LEFT JOIN by_role br ON br.oracle_id = sd.oracle_id
         LEFT JOIN other ot ON ot.oracle_id = sd.oracle_id
         WHERE cp.scryfall_data_id = sd.id",
        role_membership_ctes(1)
    );
    let result = binds
        .bind(sqlx::query(sqlx::AssertSqlSafe(sql)))
        .bind(&noise)
        .execute(pool)
        .await
        .context("failed to refresh oracle_tags_by_role / other_oracle_tags")?;
    Ok(result.rows_affected())
}
