//! Auth HTTP flows through the real router: register → authed request → login →
//! refresh (token rotation). Proves the whole stack (handler + middleware +
//! service + repo + real SQL) works end-to-end in a test.
//!
//! Requires `DATABASE_URL` (dev's value works): `set -a; source zerver/.env`.

#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

mod common;

use axum::http::StatusCode;
use common::TestApp;

#[sqlx::test]
async fn register_authed_login_refresh(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);

    // 1. Register — 201 + a session.
    let (token, user_id) = app.register("alice").await;
    assert!(!token.is_empty());

    // 2. Authed request with the access token.
    let (status, me) = app.get("/api/user", Some(&token)).await;
    assert_eq!(status, StatusCode::OK, "GET /api/user: {me}");
    assert_eq!(me["username"], "alice");

    // 3. Login with the same credentials.
    let (status, session) = app
        .post(
            "/api/auth/login",
            serde_json::json!({ "identifier": "alice", "password": "TestPass123!" }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "login: {session}");
    let refresh = session["refresh_token"]["value"]
        .as_str()
        .unwrap()
        .to_string();

    // 4. Refresh — old token rotates into a new session.
    let (status, refreshed) = app
        .post(
            "/api/auth/refresh",
            serde_json::json!({ "user_id": user_id, "refresh_token": refresh }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "refresh: {refreshed}");
    assert!(refreshed["access_token"]["value"].as_str().is_some());
}

#[sqlx::test]
async fn login_wrong_password_is_401(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    app.register("bob").await;

    let (status, _) = app
        .post(
            "/api/auth/login",
            serde_json::json!({ "identifier": "bob", "password": "WrongPass123!" }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn authed_route_without_token_is_401(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (status, _) = app.get("/api/user", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
