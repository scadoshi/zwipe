//! Per-session client-version recording through the real router: login stamps
//! `refresh_tokens.client_version`, refresh **overwrites** it with the version
//! the client re-sends (versions change on app update), and a refresh that
//! omits it carries the stored value forward.
//!
//! Requires `DATABASE_URL` (dev's value works): `set -a; source zerver/.env`.

#![expect(
    clippy::unwrap_used,
    reason = "a test: helpers outside #[test] fns unwrap, and a panic is the failure report"
)]

use zwipe_core::http::paths::{LOGIN_ROUTE, REFRESH_SESSION_ROUTE, REGISTER_ROUTE};
mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::json;
use uuid::Uuid;

/// The `client_version` on the user's single live refresh-token row.
async fn stored_version(app: &TestApp, user_id: &str) -> Option<String> {
    let uid = Uuid::parse_str(user_id).unwrap();
    sqlx::query_scalar(
        "SELECT client_version FROM refresh_tokens WHERE user_id = $1 AND replaced_at IS NULL",
    )
    .bind(uid)
    .fetch_one(&app.pool)
    .await
    .unwrap()
}

/// Registers `username` and logs in once reporting `1.6.1`, so the user has
/// a single live session carrying that version. Returns the login's session.
async fn register_and_login_at_1_6_1(app: &TestApp, username: &str) -> serde_json::Value {
    let (status, body) = app
        .post(
            REGISTER_ROUTE,
            json!({
                "username": username,
                "email": format!("{username}@test.local"),
                "password": "TestPass123!",
            }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::ACCEPTED, "register: {body}");
    let (status, session) = app
        .post(
            LOGIN_ROUTE,
            json!({
                "identifier": username,
                "password": "TestPass123!",
                "client_version": "1.6.1",
            }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "login: {session}");
    session
}

#[sqlx::test]
async fn login_stamps_client_version(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    // The helper's login sends no version → that row stays NULL.
    let (_token, user_id) = app.register("ver_login").await;

    let (status, session) = app
        .post(
            LOGIN_ROUTE,
            json!({
                "identifier": "ver_login",
                "password": "TestPass123!",
                "client_version": "1.6.2",
            }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "login: {session}");

    // Two sessions now exist; only the login row carries a version.
    let uid = Uuid::parse_str(&user_id).unwrap();
    let versioned: Option<String> = sqlx::query_scalar(
        "SELECT client_version FROM refresh_tokens WHERE user_id = $1 AND client_version IS NOT NULL",
    )
    .bind(uid)
    .fetch_one(&app.pool)
    .await
    .unwrap();
    assert_eq!(versioned.as_deref(), Some("1.6.2"));
}

#[sqlx::test]
async fn refresh_overwrites_client_version(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let session = register_and_login_at_1_6_1(&app, "ver_ovr").await;
    let user_id = session["user"]["id"].as_str().unwrap().to_string();
    let refresh = session["refresh_token"]["value"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(
        stored_version(&app, &user_id).await.as_deref(),
        Some("1.6.1")
    );

    // The client updated and now reports a newer version on refresh.
    let (status, rotated) = app
        .post(
            REFRESH_SESSION_ROUTE,
            json!({ "user_id": user_id, "refresh_token": refresh, "client_version": "9.9.9" }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "refresh: {rotated}");
    assert_eq!(
        stored_version(&app, &user_id).await.as_deref(),
        Some("9.9.9")
    );
}

#[sqlx::test]
async fn refresh_without_version_carries_forward(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let session = register_and_login_at_1_6_1(&app, "ver_carry").await;
    let user_id = session["user"]["id"].as_str().unwrap().to_string();
    let refresh = session["refresh_token"]["value"]
        .as_str()
        .unwrap()
        .to_string();

    // Older client omits the field → the stored version survives rotation.
    let (status, rotated) = app
        .post(
            REFRESH_SESSION_ROUTE,
            json!({ "user_id": user_id, "refresh_token": refresh }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "refresh: {rotated}");
    assert_eq!(
        stored_version(&app, &user_id).await.as_deref(),
        Some("1.6.1")
    );
}
