//! Usage flushes through the real router land in `platform_daily_usage` under
//! the platform carried on the caller's access token: set at login, kept
//! across a refresh, `unknown` for a token without one, and counted after the
//! swipe cap.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "a test: helpers outside #[test] fns unwrap and index, and a panic is the failure report"
)]

mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::{Value, json};
use zwipe_core::http::contracts::metrics::HttpUsageBatch;
use zwipe_core::http::paths::{RECORD_USAGE_ROUTE, REFRESH_SESSION_ROUTE};

/// One day's row for a platform: the four swipe directions, searches and flushes.
type UsageRow = (i64, i64, i64, i64, i64, i64);

/// Today's (UTC) row for `platform`, if one exists.
async fn usage_row(pool: &sqlx::PgPool, platform: &str) -> Option<UsageRow> {
    sqlx::query_as(
        "SELECT swipes_right, swipes_left, swipes_up, swipes_down, searches, flushes \
         FROM platform_daily_usage \
         WHERE day = (NOW() AT TIME ZONE 'UTC')::date AND platform = $1",
    )
    .bind(platform)
    .fetch_optional(pool)
    .await
    .unwrap()
}

/// Posts a usage batch with the given counts and expects it accepted.
async fn flush(app: &TestApp, token: &str, batch: Value) {
    let (status, body) = app.post(RECORD_USAGE_ROUTE, batch, Some(token)).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "usage flush: {body}");
}

fn access_token(session: &Value) -> &str {
    session["access_token"]["value"].as_str().unwrap()
}

#[sqlx::test]
async fn flushes_land_under_the_login_platform(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let ios = app.register_on("iosuser", Some("ios")).await;
    let terminal = app.register_on("termuser", Some("terminal")).await;

    flush(
        &app,
        access_token(&ios),
        json!({ "swipes_right": 3, "swipes_left": 1, "swipes_up": 0, "swipes_down": 2, "searches": 4 }),
    )
    .await;
    flush(
        &app,
        access_token(&terminal),
        json!({ "swipes_right": 0, "swipes_left": 5, "swipes_up": 1, "swipes_down": 0, "searches": 0 }),
    )
    .await;

    assert_eq!(usage_row(&pool, "ios").await, Some((3, 1, 0, 2, 4, 1)));
    assert_eq!(usage_row(&pool, "terminal").await, Some((0, 5, 1, 0, 0, 1)));
    assert_eq!(usage_row(&pool, "unknown").await, None);
}

#[sqlx::test]
async fn two_flushes_on_one_day_add_up(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let session = app.register_on("twiceuser", Some("android")).await;
    let token = access_token(&session);

    flush(
        &app,
        token,
        json!({ "swipes_right": 2, "swipes_left": 1, "swipes_up": 0, "swipes_down": 0, "searches": 1 }),
    )
    .await;
    flush(
        &app,
        token,
        json!({ "swipes_right": 3, "swipes_left": 0, "swipes_up": 1, "swipes_down": 1, "searches": 2 }),
    )
    .await;

    assert_eq!(usage_row(&pool, "android").await, Some((5, 1, 1, 1, 3, 2)));
}

#[sqlx::test]
async fn a_refreshed_session_keeps_its_platform(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let session = app.register_on("refreshuser", Some("ios")).await;

    let (status, refreshed) = app
        .post(
            REFRESH_SESSION_ROUTE,
            json!({
                "user_id": session["user"]["id"],
                "refresh_token": session["refresh_token"]["value"],
            }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "refresh: {refreshed}");

    flush(
        &app,
        access_token(&refreshed),
        json!({ "swipes_right": 1, "swipes_left": 0, "swipes_up": 0, "swipes_down": 0, "searches": 0 }),
    )
    .await;

    assert_eq!(usage_row(&pool, "ios").await, Some((1, 0, 0, 0, 0, 1)));
    assert_eq!(usage_row(&pool, "unknown").await, None);
}

#[sqlx::test]
async fn a_token_without_a_platform_lands_under_unknown(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, _) = app.register("noplatform").await;

    flush(
        &app,
        &token,
        json!({ "swipes_right": 0, "swipes_left": 2, "swipes_up": 0, "swipes_down": 0, "searches": 1 }),
    )
    .await;

    assert_eq!(usage_row(&pool, "unknown").await, Some((0, 2, 0, 0, 1, 1)));
}

#[sqlx::test]
async fn swipes_over_the_cap_land_capped(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let session = app.register_on("capuser", Some("web")).await;
    let over = HttpUsageBatch::MAX_SWIPES_PER_FLUSH + 50;

    flush(
        &app,
        access_token(&session),
        json!({ "swipes_right": over, "swipes_left": 10, "swipes_up": 0, "swipes_down": 0, "searches": 0 }),
    )
    .await;

    let cap = i64::from(HttpUsageBatch::MAX_SWIPES_PER_FLUSH);
    assert_eq!(usage_row(&pool, "web").await, Some((cap, 0, 0, 0, 0, 1)));
}
