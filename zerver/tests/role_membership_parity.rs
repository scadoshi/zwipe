//! zerver's SQL role membership (`role_membership_ctes`) and zwipe-core's
//! `RoleMembership::from_hierarchy`, which the synergy worker runs, give the same
//! role-to-slug membership over one seeded `oracle_tags` hierarchy: role roots
//! with nested children, a tag with parents under two roles, an exclusion slug
//! inside a subtree with a child of its own, override slugs both in and out of
//! the catalog, a slug that is both a root and an override, a cycle, an orphan
//! and a tag under no role.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::collections::{BTreeMap, BTreeSet};

use uuid::Uuid;
use zwipe::outbound::sqlx::card::helpers::oracle_tag_groups::{
    RoleMembershipBinds, role_membership_ctes,
};
use zwipe_core::domain::card::card_role::role_tags::{OracleTagNode, RoleMembership};

/// Writes one catalog row with the given parent ids.
async fn seed_tag(pool: &sqlx::PgPool, id: u128, slug: &str, parents: &[u128]) {
    let parents: Vec<Uuid> = parents.iter().copied().map(Uuid::from_u128).collect();
    sqlx::query("INSERT INTO oracle_tags (id, slug, label, parent_ids) VALUES ($1, $2, $2, $3)")
        .bind(Uuid::from_u128(id))
        .bind(slug)
        .bind(&parents)
        .execute(pool)
        .await
        .unwrap();
}

/// Runs the SQL membership CTE and groups its rows by role.
async fn sql_membership(pool: &sqlx::PgPool) -> BTreeMap<String, BTreeSet<String>> {
    let binds = RoleMembershipBinds::new();
    let sql = format!(
        "WITH RECURSIVE {} SELECT role, slug FROM membership",
        role_membership_ctes(1)
    );
    let rows: Vec<(String, String)> = binds
        .bind(sqlx::query(sqlx::AssertSqlSafe(sql)))
        .fetch_all(pool)
        .await
        .unwrap()
        .into_iter()
        .map(|row| {
            use sqlx::Row;
            (row.get("role"), row.get("slug"))
        })
        .collect();
    let mut by_role: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (role, slug) in rows {
        by_role.entry(role).or_default().insert(slug);
    }
    by_role
}

/// Reads the hierarchy the way the synergy worker does and runs core's rules.
async fn core_membership(pool: &sqlx::PgPool) -> RoleMembership {
    let rows: Vec<(Uuid, String, Vec<Uuid>)> =
        sqlx::query_as("SELECT id, slug, parent_ids FROM oracle_tags")
            .fetch_all(pool)
            .await
            .unwrap();
    let tags: Vec<OracleTagNode> = rows
        .into_iter()
        .map(|(id, slug, parent_ids)| OracleTagNode {
            id,
            slug,
            parent_ids,
        })
        .collect();
    RoleMembership::from_hierarchy(&tags)
}

#[sqlx::test]
async fn sql_and_core_role_membership_match(pool: sqlx::PgPool) {
    // removal root, a nested chain, and the burn-self exclusion with a child.
    seed_tag(&pool, 0x01, "removal", &[]).await;
    seed_tag(&pool, 0x02, "spot-removal", &[0x01]).await;
    seed_tag(&pool, 0x03, "burn-creature", &[0x02]).await;
    seed_tag(&pool, 0x04, "burn-self", &[0x03]).await;
    seed_tag(&pool, 0x05, "burn-self-child", &[0x04]).await;
    // ramp root; a tag under both ramp and removal, with a child.
    seed_tag(&pool, 0x10, "ramp", &[]).await;
    seed_tag(&pool, 0x11, "ramp-removal", &[0x10, 0x02]).await;
    seed_tag(&pool, 0x12, "ramp-removal-child", &[0x11]).await;
    // untap root with the twiddle exclusion as a direct child.
    seed_tag(&pool, 0x20, "untapper", &[]).await;
    seed_tag(&pool, 0x21, "twiddle", &[0x20]).await;
    // mill reached through surveil, repeatable-maps excluded but also under
    // tokens through a second parent.
    seed_tag(&pool, 0x30, "mill", &[]).await;
    seed_tag(&pool, 0x31, "surveil", &[0x30]).await;
    seed_tag(&pool, 0x32, "synergy-token", &[]).await;
    seed_tag(&pool, 0x33, "repeatable-maps", &[0x31, 0x32]).await;
    // An override in the catalog with a child the override must not pull in.
    seed_tag(&pool, 0x40, "pinger", &[]).await;
    seed_tag(&pool, 0x41, "pinger-child", &[0x40]).await;
    // mass-land-denial: a wipe root and a removal and stax override.
    seed_tag(&pool, 0x50, "mass-land-denial", &[]).await;
    // A cycle under the tutor root.
    seed_tag(&pool, 0x60, "tutor", &[0x62]).await;
    seed_tag(&pool, 0x61, "tutor-a", &[0x60]).await;
    seed_tag(&pool, 0x62, "tutor-b", &[0x61]).await;
    // Under no role, and an orphan naming a missing parent.
    seed_tag(&pool, 0x70, "typal-goblin", &[]).await;
    seed_tag(&pool, 0x71, "orphan", &[0xff]).await;

    let sql = sql_membership(&pool).await;
    let core = core_membership(&pool).await;
    assert_eq!(&sql, core.by_role(), "sql vs core membership");

    // Pin the cases the seed is built around, so a shared mistake fails too.
    let roles = |slug: &str| -> Vec<&str> {
        core.roles_of(slug)
            .map(|r| r.iter().map(String::as_str).collect())
            .unwrap_or_default()
    };
    assert_eq!(roles("burn-creature"), ["removal"]);
    assert!(roles("burn-self").is_empty(), "excluded from removal");
    assert_eq!(
        roles("burn-self-child"),
        ["removal"],
        "exclusions are exact"
    );
    assert_eq!(roles("ramp-removal-child"), ["ramp", "removal"]);
    assert!(roles("twiddle").is_empty());
    assert_eq!(roles("repeatable-maps"), ["tokens"]);
    assert_eq!(roles("pinger"), ["burn"]);
    assert!(roles("pinger-child").is_empty(), "overrides never expand");
    assert_eq!(roles("mass-land-denial"), ["removal", "stax", "wipe"]);
    assert_eq!(roles("tutor-b"), ["tutor"]);
    assert_eq!(roles("hatebear"), ["stax"], "override outside the catalog");
    assert!(roles("typal-goblin").is_empty() && roles("orphan").is_empty());
}
