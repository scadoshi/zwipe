//! Health + root endpoints through the real router. `/health/database` pings
//! Postgres, so this also proves the test pool wiring end to end.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![allow(clippy::unwrap_used)]

mod common;

use axum::http::StatusCode;
use common::TestApp;

#[sqlx::test]
async fn health_and_root_ok(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    for path in ["/", "/health", "/health/server", "/health/database"] {
        let (status, _) = app.get(path, None).await;
        assert_eq!(status, StatusCode::OK, "GET {path}");
    }
}
