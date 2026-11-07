//! Refresh token families through the real router: a rotation keeps the old
//! row as replay evidence and carries the family forward, a token presented
//! again inside the reuse window gets the same successor (two tabs racing),
//! a replay past the window deletes the whole family (the fresh token from
//! that rotation dies with it), a family older than the absolute lifespan
//! cannot rotate, concurrent refreshes of one token all succeed with one
//! shared successor, and a replaced row outlives its own expiry so a late
//! replay is still caught.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "a test: helpers outside #[test] fns unwrap and index, and a panic is the failure report"
)]

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

/// Ages every replaced row of the user past the reuse window.
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
async fn a_second_presentation_inside_the_window_gets_the_same_successor(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (user_id, token) = login(&app, "second-tab").await;

    let (status, first) = refresh(&app, &user_id, &token).await;
    assert_eq!(status, StatusCode::OK, "first refresh: {first}");
    let (status, second) = refresh(&app, &user_id, &token).await;
    assert_eq!(status, StatusCode::OK, "the racing second tab: {second}");
    let successor = refresh_value(&first);
    assert_eq!(
        refresh_value(&second),
        successor,
        "both tabs hold one token, so the family keeps one head"
    );

    // The family lives: register, the rotated login row and one successor.
    let rows = rows(&app, &user_id).await;
    assert_eq!(
        rows.len(),
        3,
        "nothing was revoked or minted twice: {rows:?}"
    );

    // The shared successor rotates as any token does.
    let (status, next) = refresh(&app, &user_id, &successor).await;
    assert_eq!(status, StatusCode::OK, "the successor rotates: {next}");
}

#[sqlx::test]
async fn a_lost_response_then_the_old_token_again_gets_the_newest_successor(pool: sqlx::PgPool) {
    // One tab, full page loads: a refresh rotates on the server but the page
    // unloads before the browser applies the reply, so the next load sends
    // the old token again.
    let app = TestApp::new(pool);
    let (user_id, first) = login(&app, "reloader").await;

    let (status, lost) = refresh(&app, &user_id, &first).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "the rotation whose reply is lost: {lost}"
    );
    let second = refresh_value(&lost);

    // The next load presents the old token: it gets that same successor,
    // twice over if that reply is lost too.
    for _ in 0..2 {
        let (status, again) = refresh(&app, &user_id, &first).await;
        assert_eq!(
            status,
            StatusCode::OK,
            "the old token inside the window: {again}"
        );
        assert_eq!(refresh_value(&again), second, "the same successor");
    }

    // A load that kept the successor rotates it, and that reply is lost too;
    // the oldest token still lands on the family's newest one.
    let (status, rotated) = refresh(&app, &user_id, &second).await;
    assert_eq!(status, StatusCode::OK, "the successor rotates: {rotated}");
    let third = refresh_value(&rotated);
    let (status, again) = refresh(&app, &user_id, &second).await;
    assert_eq!(status, StatusCode::OK, "the lost successor again: {again}");
    assert_eq!(refresh_value(&again), third);
    let (status, again) = refresh(&app, &user_id, &first).await;
    assert_eq!(status, StatusCode::OK, "the oldest token again: {again}");
    assert_eq!(
        refresh_value(&again),
        third,
        "the newest successor, following the chain"
    );

    // The family lives, with one live head, and that head rotates.
    let rows = rows(&app, &user_id).await;
    let live = rows.iter().filter(|(_, replaced)| !*replaced).count();
    assert_eq!(live, 2, "the register session and the one head: {rows:?}");
    let (status, _) = refresh(&app, &user_id, &third).await;
    assert_eq!(status, StatusCode::OK, "the head rotates");
}

#[sqlx::test]
async fn replay_past_the_window_kills_the_family(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (user_id, token) = login(&app, "replayed").await;

    let (status, rotated) = refresh(&app, &user_id, &token).await;
    assert_eq!(status, StatusCode::OK, "first refresh: {rotated}");
    let fresh = refresh_value(&rotated);

    // A minute later: past the reuse window, the old token is a replay.
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
async fn concurrent_refreshes_share_one_successor(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (user_id, token) = login(&app, "racer").await;

    let (a, b, c, d) = tokio::join!(
        refresh(&app, &user_id, &token),
        refresh(&app, &user_id, &token),
        refresh(&app, &user_id, &token),
        refresh(&app, &user_id, &token),
    );
    let results = [a, b, c, d];
    assert!(
        results.iter().all(|(status, _)| *status == StatusCode::OK),
        "every racing refresh succeeds: {results:?}"
    );
    // One rotation won; the others, inside the reuse window, got its
    // successor rather than a second branch, so the family has one head.
    let survivor = refresh_value(&results[0].1);
    assert!(
        results
            .iter()
            .all(|(_, body)| refresh_value(body) == survivor),
        "all four share one successor: {results:?}"
    );
    let rows = rows(&app, &user_id).await;
    let live = rows.iter().filter(|(_, replaced)| !*replaced).count();
    assert_eq!(
        live, 2,
        "the register session and the one successor: {rows:?}"
    );

    let (status, _) = refresh(&app, &user_id, &survivor).await;
    assert_eq!(status, StatusCode::OK, "the shared successor rotates");
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
    let repo = Postgres::with_pool(pool.clone());
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
