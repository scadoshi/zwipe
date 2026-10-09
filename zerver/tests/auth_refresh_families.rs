//! Refresh token families through the real router: a rotation keeps the old
//! row as replay evidence and carries the family forward, a replay past the
//! grace window deletes the whole family (the fresh token from that rotation
//! dies with it), a family older than the absolute lifespan cannot rotate, and
//! concurrent refreshes of one token still yield one winner whose token works.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use zwipe_core::http::paths::*;
mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::{Value, json};
use uuid::Uuid;

/// Registers `username` and returns the session's user id and refresh token.
async fn login(app: &TestApp, username: &str) -> (String, String) {
    let _ = app.register(username).await;
    let (status, session) = app
        .post(
            LOGIN_ROUTE,
            json!({ "identifier": username, "password": "TestPass123!" }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "login: {session}");
    (
        session["user"]["id"].as_str().unwrap().to_string(),
        session["refresh_token"]["value"]
            .as_str()
            .unwrap()
            .to_string(),
    )
}

async fn refresh(app: &TestApp, user_id: &str, token: &str) -> (StatusCode, Value) {
    app.post(
        REFRESH_SESSION_ROUTE,
        json!({ "user_id": user_id, "refresh_token": token }),
        None,
    )
    .await
}

fn refresh_value(session: &Value) -> String {
    session["refresh_token"]["value"]
        .as_str()
        .unwrap()
        .to_string()
}

/// `(family_id, replaced_at IS NOT NULL)` for every row of the user, oldest first.
async fn rows(app: &TestApp, user_id: &str) -> Vec<(Uuid, bool)> {
    let uid = Uuid::parse_str(user_id).unwrap();
    sqlx::query_as(
        "SELECT family_id, replaced_at IS NOT NULL FROM refresh_tokens WHERE user_id = $1 ORDER BY created_at",
    )
    .bind(uid)
    .fetch_all(&app.pool)
    .await
    .unwrap()
}

/// Ages every replaced row of the user past the concurrent-rotation grace window.
async fn age_replaced_rows(app: &TestApp, user_id: &str) {
    let uid = Uuid::parse_str(user_id).unwrap();
    sqlx::query(
        "UPDATE refresh_tokens SET replaced_at = replaced_at - INTERVAL '1 minute' WHERE user_id = $1 AND replaced_at IS NOT NULL",
    )
    .bind(uid)
    .execute(&app.pool)
    .await
    .unwrap();
}

#[sqlx::test]
async fn rotation_keeps_replaced_row_in_the_same_family(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (user_id, token) = login(&app, "lineage").await;

    let (status, rotated) = refresh(&app, &user_id, &token).await;
    assert_eq!(status, StatusCode::OK, "refresh: {rotated}");

    // register + login rows, then the rotation's successor
    let rows = rows(&app, &user_id).await;
    assert_eq!(rows.len(), 3, "the rotated row stays: {rows:?}");
    let (register_family, register_replaced) = rows[0];
    let (login_family, login_replaced) = rows[1];
    let (new_family, new_replaced) = rows[2];
    assert!(!register_replaced, "the untouched register session is live");
    assert!(login_replaced, "the rotated login token is marked replaced");
    assert!(!new_replaced, "the successor is live");
    assert_eq!(login_family, new_family, "rotation carries the family");
    assert_ne!(register_family, login_family, "each login starts a family");
}

#[sqlx::test]
async fn replay_kills_the_family(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (user_id, token) = login(&app, "replayed").await;

    let (status, rotated) = refresh(&app, &user_id, &token).await;
    assert_eq!(status, StatusCode::OK, "first refresh: {rotated}");
    let fresh = refresh_value(&rotated);
    age_replaced_rows(&app, &user_id).await;

    let (status, _) = refresh(&app, &user_id, &token).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "the replay is refused");

    let (status, _) = refresh(&app, &user_id, &fresh).await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "the fresh token from that rotation died with the family"
    );

    let rows = rows(&app, &user_id).await;
    assert_eq!(rows.len(), 1, "only the register session remains: {rows:?}");
    assert!(!rows[0].1, "and it is live");
}

#[sqlx::test]
async fn family_past_absolute_lifespan_cannot_rotate(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (user_id, token) = login(&app, "longhaul").await;
    let uid = Uuid::parse_str(&user_id).unwrap();

    // The login happened 31 days ago; the token itself is still inside its 14 days.
    sqlx::query(
        "UPDATE refresh_tokens SET login_at = NOW() - INTERVAL '31 days' WHERE user_id = $1",
    )
    .bind(uid)
    .execute(&app.pool)
    .await
    .unwrap();

    let (status, _) = refresh(&app, &user_id, &token).await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "a 31-day-old family is expired"
    );

    let rows = rows(&app, &user_id).await;
    assert_eq!(rows.len(), 1, "the expired family is gone: {rows:?}");
}

#[sqlx::test]
async fn concurrent_refreshes_leave_one_valid_survivor(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (user_id, token) = login(&app, "racer").await;

    let (a, b, c, d) = tokio::join!(
        refresh(&app, &user_id, &token),
        refresh(&app, &user_id, &token),
        refresh(&app, &user_id, &token),
        refresh(&app, &user_id, &token),
    );
    let results = [a, b, c, d];
    let winners: Vec<&Value> = results
        .iter()
        .filter(|(status, _)| *status == StatusCode::OK)
        .map(|(_, body)| body)
        .collect();
    let losers = results
        .iter()
        .filter(|(status, _)| *status == StatusCode::UNAUTHORIZED)
        .count();
    assert_eq!(winners.len(), 1, "exactly one refresh wins: {results:?}");
    assert_eq!(losers, 3, "the other three are 401: {results:?}");

    // The losers were concurrent duplicates, not a replay: the family lives.
    let survivor = refresh_value(winners[0]);
    let (status, rotated) = refresh(&app, &user_id, &survivor).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "the survivor still rotates: {rotated}"
    );
}
