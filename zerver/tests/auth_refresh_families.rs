//! Refresh token families through the real router: a rotation keeps the old
//! row as replay evidence and carries the family forward, a replay past the
//! grace window deletes the whole family (the fresh token from that rotation
//! dies with it), a family older than the absolute lifespan cannot rotate,
//! concurrent refreshes of one token still yield one winner whose token works,
//! and a replaced row outlives its own expiry so a late replay is still caught.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use zwipe_core::http::paths::{LOGIN_ROUTE, REFRESH_SESSION_ROUTE};
mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::{Value, json};
use uuid::Uuid;
use zwipe::domain::upkeep::ports::UpkeepRepository;
use zwipe::domain::upkeep::services::REPLACED_TOKEN_RETENTION_DAYS;
use zwipe::outbound::sqlx::postgres::Postgres;

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

    // Straight away, with no ageing: there is no grace window a replay can
    // hide inside.
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

    // Each loser presented an already-rotated token, which is a replay: the
    // family is gone, the winner's token with it, and the device logs in
    // again. The client refreshes single-flight, so this only happens when
    // a token has a second holder.
    let survivor = refresh_value(winners[0]);
    let (status, _) = refresh(&app, &user_id, &survivor).await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "the winner's token died with the family"
    );
    let rows = rows(&app, &user_id).await;
    assert_eq!(rows.len(), 1, "only the register session remains: {rows:?}");
}

#[sqlx::test]
async fn replaced_row_outlives_its_expiry_as_replay_evidence(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (user_id, token) = login(&app, "latecomer").await;
    let uid = Uuid::parse_str(&user_id).unwrap();

    let (status, rotated) = refresh(&app, &user_id, &token).await;
    assert_eq!(status, StatusCode::OK, "refresh: {rotated}");
    let fresh = refresh_value(&rotated);

    // The token was rotated an hour ago, near the end of its own 14 days,
    // which have since run out. Its replay evidence is still inside the
    // retention window and must survive both prunes.
    sqlx::query(
        "UPDATE refresh_tokens
         SET expires_at = NOW() - INTERVAL '1 day', replaced_at = NOW() - INTERVAL '1 hour'
         WHERE user_id = $1 AND replaced_at IS NOT NULL",
    )
    .bind(uid)
    .execute(&app.pool)
    .await
    .unwrap();

    // The nightly sweep, then the insert-time prune that a fresh login runs.
    let repo = Postgres { pool: pool.clone() };
    let pruned = repo
        .prune_expired_sessions(REPLACED_TOKEN_RETENTION_DAYS)
        .await
        .unwrap();
    assert_eq!(pruned, 0, "the replaced row is kept past its expiry");
    let (status, session) = app
        .post(
            LOGIN_ROUTE,
            json!({ "identifier": "latecomer", "password": "TestPass123!" }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "second login: {session}");
    let second_login = refresh_value(&session);

    let kept = rows(&app, &user_id).await;
    assert_eq!(
        kept.len(),
        4,
        "register, replaced login, successor, second login: {kept:?}"
    );
    assert!(kept[1].1, "the replaced row still stands as evidence");

    let (status, _) = refresh(&app, &user_id, &token).await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "the late replay is refused"
    );
    let (status, _) = refresh(&app, &user_id, &fresh).await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "the fresh token from that rotation died with the family"
    );

    let rows = rows(&app, &user_id).await;
    assert_eq!(
        rows.len(),
        2,
        "the register and second-login sessions remain: {rows:?}"
    );
    assert!(
        rows.iter().all(|(_, replaced)| !replaced),
        "and both are live"
    );

    let (status, _) = refresh(&app, &user_id, &second_login).await;
    assert_eq!(status, StatusCode::OK, "the unrelated family still rotates");
}
