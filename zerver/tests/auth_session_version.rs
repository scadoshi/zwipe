//! Per-session client-version recording through the real router: register and
//! login stamp `refresh_tokens.client_version`, refresh **overwrites** it with
//! the version the client re-sends (versions change on app update), and a
//! refresh that omits it carries the stored value forward.
//!
//! Requires `DATABASE_URL` (dev's value works): `set -a; source zerver/.env`.

#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::json;
use uuid::Uuid;

/// The `client_version` on the user's single refresh-token row.
async fn stored_version(app: &TestApp, user_id: &str) -> Option<String> {
    let uid = Uuid::parse_str(user_id).unwrap();
    sqlx::query_scalar("SELECT client_version FROM refresh_tokens WHERE user_id = $1")
        .bind(uid)
        .fetch_one(&app.pool)
        .await
        .unwrap()
}

#[sqlx::test]
async fn register_stamps_client_version(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (status, session) = app
        .post(
            "/api/auth/register",
            json!({
                "username": "ver_reg",
                "email": "ver_reg@test.local",
                "password": "TestPass123!",
                "client_version": "1.6.1",
            }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "register: {session}");
    let user_id = session["user"]["id"].as_str().unwrap();
    assert_eq!(
        stored_version(&app, user_id).await.as_deref(),
        Some("1.6.1")
    );
}

#[sqlx::test]
async fn login_stamps_client_version(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    // Helper register sends no version → that row stays NULL.
    let (_token, user_id) = app.register("ver_login").await;

    let (status, session) = app
        .post(
            "/api/auth/login",
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
    let (status, session) = app
        .post(
            "/api/auth/register",
            json!({
                "username": "ver_ovr",
                "email": "ver_ovr@test.local",
                "password": "TestPass123!",
                "client_version": "1.6.1",
            }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "register: {session}");
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
            "/api/auth/refresh",
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
    let (status, session) = app
        .post(
            "/api/auth/register",
            json!({
                "username": "ver_carry",
                "email": "ver_carry@test.local",
                "password": "TestPass123!",
                "client_version": "1.6.1",
            }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "register: {session}");
    let user_id = session["user"]["id"].as_str().unwrap().to_string();
    let refresh = session["refresh_token"]["value"]
        .as_str()
        .unwrap()
        .to_string();

    // Older client omits the field → the stored version survives rotation.
    let (status, rotated) = app
        .post(
            "/api/auth/refresh",
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
