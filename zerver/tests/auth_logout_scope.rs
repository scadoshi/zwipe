//! Logging out ends the session that asks and leaves the user's other
//! sessions alone. An access token minted without a `family_id` claim names
//! no session, so its logout ends every session the user holds.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "a test: helpers unwrap and index, and a panic is the failure report"
)]

mod common;

use axum::http::StatusCode;
use chrono::Utc;
use common::{TEST_JWT_SECRET, TestApp};
use jsonwebtoken::{EncodingKey, Header};
use serde_json::{Value, json};
use zwipe_core::http::paths::{LOGIN_ROUTE, LOGOUT_ROUTE, REFRESH_SESSION_ROUTE};

/// A second login for `username`, as another device makes it.
async fn login(app: &TestApp, username: &str) -> Value {
    let (status, session) = app
        .post(
            LOGIN_ROUTE,
            json!({ "identifier": username, "password": "TestPass123!" }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "login: {session}");
    session
}

fn access(session: &Value) -> &str {
    session["access_token"]["value"].as_str().unwrap()
}

/// Spends `session`'s refresh token and answers with the status.
async fn refresh(app: &TestApp, session: &Value) -> StatusCode {
    let body = json!({
        "user_id": session["user"]["id"],
        "refresh_token": session["refresh_token"]["value"],
    });
    app.post(REFRESH_SESSION_ROUTE, body, None).await.0
}

async fn logout(app: &TestApp, token: &str) {
    let (status, body) = app.post(LOGOUT_ROUTE, Value::Null, Some(token)).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "logout: {body}");
}

#[sqlx::test]
async fn logging_out_one_device_keeps_the_other_signed_in(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let phone = app.register_on("twodevices", Some("android")).await;
    let laptop = login(&app, "twodevices").await;

    logout(&app, access(&phone)).await;

    assert_eq!(
        refresh(&app, &laptop).await,
        StatusCode::OK,
        "the other device's session survives"
    );
    assert_eq!(
        refresh(&app, &phone).await,
        StatusCode::UNAUTHORIZED,
        "the device that logged out is signed out"
    );
}

#[sqlx::test]
async fn a_token_without_a_family_logs_out_every_device(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let phone = app.register_on("oldtoken", Some("android")).await;
    let laptop = login(&app, "oldtoken").await;

    let now = Utc::now().timestamp();
    let claims = json!({
        "user_id": phone["user"]["id"],
        "username": phone["user"]["username"],
        "email": phone["user"]["email"],
        "iat": now,
        "exp": now.saturating_add(3600),
    });
    let familyless = jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(TEST_JWT_SECRET.as_bytes()),
    )
    .unwrap();

    logout(&app, &familyless).await;

    assert_eq!(refresh(&app, &laptop).await, StatusCode::UNAUTHORIZED);
    assert_eq!(refresh(&app, &phone).await, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn a_refreshed_token_still_names_its_own_session(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let phone = app.register_on("refreshedout", Some("ios")).await;
    let laptop = login(&app, "refreshedout").await;
    let body = json!({
        "user_id": phone["user"]["id"],
        "refresh_token": phone["refresh_token"]["value"],
    });
    let (status, rotated) = app.post(REFRESH_SESSION_ROUTE, body, None).await;
    assert_eq!(status, StatusCode::OK, "refresh: {rotated}");

    logout(&app, access(&rotated)).await;

    assert_eq!(
        refresh(&app, &laptop).await,
        StatusCode::OK,
        "the other device's session survives"
    );
    assert_eq!(refresh(&app, &rotated).await, StatusCode::UNAUTHORIZED);
}
