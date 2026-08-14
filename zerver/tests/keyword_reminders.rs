//! Keyword-reminder catalog: the public endpoint serves every keyword the
//! database knows, mapped through the core reminder table — so definition
//! fixes ship on server deploy while app binaries keep the compiled fallback.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

mod common;

use axum::http::StatusCode;
use common::{TestApp, card, seed_cards};
use zwipe_core::domain::card::keyword::keyword_reminder;

#[sqlx::test]
async fn serves_reminders_for_database_keywords(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());

    let flyer = card("Sky Tester").mono("U").keywords(&["Flying", "Blight"]);
    seed_cards(&pool, &[flyer]).await;

    // Public: no auth needed.
    let (status, body) = app.get("/api/card/keyword-reminders", None).await;
    assert_eq!(status, StatusCode::OK, "catalog get: {body}");

    // Keys are lowercase — the catalog query normalizes names, and the chips
    // component lowercases its lookups to match.
    let map = body.as_object().unwrap();
    assert_eq!(
        map["flying"].as_str().unwrap(),
        keyword_reminder("Flying"),
        "served text matches the core table"
    );
    assert_eq!(
        map["blight"].as_str().unwrap(),
        keyword_reminder("Blight"),
        "new-set keywords ride the same table"
    );
}
