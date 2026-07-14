//! Public oracle-tag catalog endpoint. Serves the full tag catalog (with our
//! overlaid descriptions) that the in-app oracle-tag dictionary reads. No auth —
//! the route lives under `/api/card/` so Cloudflare edge-caches it, and CF bypasses
//! cache for any request carrying `Authorization`. The **no-auth** assertion here is
//! the regression guard for that cache contract: make this route require auth and
//! cache HITs silently vanish, 100x'ing origin load.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

mod common;

use axum::http::StatusCode;
use common::TestApp;
use uuid::Uuid;

#[sqlx::test]
async fn oracle_tags_endpoint_is_public_and_returns_catalog(pool: sqlx::PgPool) {
    // Seed a parent/child pair: `removal` (described) → `spot-removal` (no description).
    let removal = Uuid::from_u128(0x1);
    let spot = Uuid::from_u128(0x2);
    sqlx::query(
        "INSERT INTO oracle_tags (id, slug, label, description) \
         VALUES ($1, 'removal', 'Removal', 'Removes stuff')",
    )
    .bind(removal)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO oracle_tags (id, slug, label, parent_ids) \
         VALUES ($1, 'spot-removal', 'Spot removal', ARRAY[$2]::uuid[])",
    )
    .bind(spot)
    .bind(removal)
    .execute(&pool)
    .await
    .unwrap();

    let app = TestApp::new(pool);
    // `None` = no Authorization header. Public + unauthenticated is required for the
    // Cloudflare cache to serve it.
    let (status, body) = app.get("/api/card/oracle-tags", None).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "public GET should 200 without auth: {body}"
    );

    let tags = body.as_array().unwrap();
    assert_eq!(tags.len(), 2, "returns the whole catalog: {body}");

    // Ordered by slug: `removal`, then `spot-removal`.
    assert_eq!(tags[0]["slug"], "removal");
    assert_eq!(tags[0]["label"], "Removal");
    assert_eq!(tags[0]["description"], "Removes stuff");
    assert!(
        tags[0]["parent_slugs"].as_array().unwrap().is_empty(),
        "a root tag has no parents"
    );

    // The undescribed child: `description` serializes as null (dictionary shows
    // "No description yet"), and its parent resolves to the parent's slug.
    assert_eq!(tags[1]["slug"], "spot-removal");
    assert!(
        tags[1]["description"].is_null(),
        "undescribed tag serializes description:null, not omitted"
    );
    assert_eq!(tags[1]["parent_slugs"][0], "removal");
}
