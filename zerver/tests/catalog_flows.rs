//! Server-driven catalog endpoints (server_driven_catalogs.md Parts B/C): the
//! card-role catalog (`GET /api/card/roles`, public) and the deck-tag catalog
//! (`GET /api/deck/tags`, authed). Both are built straight from the enums.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![allow(clippy::unwrap_used, clippy::indexing_slicing, clippy::expect_used)]

mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::json;

#[sqlx::test]
async fn card_roles_catalog_is_public_and_lists_all_roles(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    // Public — no auth token.
    let (status, body) = app.get("/api/card/roles", None).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "card roles catalog is public: {body}"
    );

    let roles = body.as_array().expect("array of roles");
    assert_eq!(roles.len(), 27, "every CardRole variant is served");

    let ramp = roles
        .iter()
        .find(|r| r.get("slug") == Some(&json!("ramp")))
        .expect("ramp present");
    assert_eq!(ramp.get("display_name"), Some(&json!("Ramp")));
    assert!(ramp.get("short_name").is_some(), "short_name for charts");
}

#[sqlx::test]
async fn deck_tags_catalog_routes_correctly_and_carries_seeds(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (token, _) = app.register("tagfetcher").await;

    // The static `/tags` route must win over `/{deck_id}` (get_deck) — a 200 here
    // proves it isn't shadowed by the param route.
    let (status, body) = app.get("/api/deck/tags", Some(&token)).await;
    assert_eq!(status, StatusCode::OK, "deck tags catalog: {body}");

    let tags = body.as_array().expect("array of deck tags");
    assert!(!tags.is_empty(), "deck tags served");
    let first = tags.first().expect("at least one tag");
    for key in ["slug", "display_name", "description", "seed_otags"] {
        assert!(first.get(key).is_some(), "tag view carries {key}");
    }
    assert!(
        first.get("seed_otags").unwrap().is_array(),
        "seed_otags is the seed map (array of slugs)"
    );
}

#[sqlx::test]
async fn deck_tags_catalog_requires_auth(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    // Under the private /deck nest — no token → unauthorized.
    let (status, _) = app.get("/api/deck/tags", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "deck tags is authed");
}
