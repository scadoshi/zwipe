//! Public changelog endpoint. Serves the compiled-in release history so clients
//! can fetch fresh entries without an app resubmit. No auth, no DB.
//!
//! Requires `DATABASE_URL` (to build the app): `set -a; source zerver/.env; set +a`.

#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

mod common;

use axum::http::StatusCode;
use common::TestApp;
use zwipe_core::content::changelog::{RELEASES, UPCOMING};

#[sqlx::test]
async fn changelog_serves_the_compiled_in_history(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);

    let (status, body) = app.get("/api/changelog", None).await;
    assert_eq!(status, StatusCode::OK, "GET /api/changelog: {body}");

    let releases = body["releases"].as_array().unwrap();
    let upcoming = body["upcoming"].as_array().unwrap();
    assert_eq!(
        releases.len(),
        RELEASES.len(),
        "release count matches source"
    );
    assert_eq!(
        upcoming.len(),
        UPCOMING.len(),
        "upcoming count matches source"
    );

    // Newest first, and each entry carries its notes.
    let latest = &releases[0];
    assert_eq!(latest["version"].as_str().unwrap(), RELEASES[0].version);
    assert_eq!(latest["date"].as_str().unwrap(), RELEASES[0].date);
    assert_eq!(
        latest["entries"].as_array().unwrap().len(),
        RELEASES[0].entries.len(),
        "latest release keeps all its bullets"
    );
}
