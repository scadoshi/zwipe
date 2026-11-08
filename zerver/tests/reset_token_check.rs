//! The reset link check: a live token answers 204, a used, expired or bogus
//! one 400, and a check never spends the token. The last test makes the call
//! through `zwipe-client` against the router served on a local port.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

mod common;

use std::net::SocketAddr;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::json;
use zwipe_client::{ClientError, ZwipeClient};
use zwipe_core::http::paths::{
    CHECK_RESET_TOKEN_ROUTE, FORGOT_PASSWORD_ROUTE, RESET_PASSWORD_ROUTE,
};

/// Registers `username`, asks for a reset and returns the raw token from the
/// captured email.
#[expect(
    clippy::expect_used,
    reason = "a test helper: no reset email fails the test that asked for one"
)]
async fn reset_token(app: &TestApp, username: &str) -> String {
    let _ = app.register(username).await;
    let (status, _) = app
        .post(
            FORGOT_PASSWORD_ROUTE,
            json!({ "email": format!("{username}@test.local") }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "forgot-password accepted");
    app.emails
        .last_token("reset")
        .expect("reset email with a /reset/ link")
}

async fn check(app: &TestApp, token: &str) -> StatusCode {
    app.post(CHECK_RESET_TOKEN_ROUTE, json!({ "token": token }), None)
        .await
        .0
}

#[sqlx::test]
async fn a_live_token_checks_out_and_stays_live(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let raw = reset_token(&app, "checker").await;

    assert_eq!(
        check(&app, &raw).await,
        StatusCode::NO_CONTENT,
        "first check"
    );
    assert_eq!(
        check(&app, &raw).await,
        StatusCode::NO_CONTENT,
        "second check"
    );

    let (status, _) = app
        .post(
            RESET_PASSWORD_ROUTE,
            json!({ "token": raw, "new_password": "Reset456!" }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "the checked token still resets");
}

#[sqlx::test]
async fn a_check_leaves_the_expiry_alone(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let raw = reset_token(&app, "patient").await;
    let expiry = || {
        sqlx::query_scalar::<_, chrono::DateTime<chrono::Utc>>(
            "SELECT expires_at FROM password_reset_tokens",
        )
        .fetch_one(&app.pool)
    };
    let before = expiry().await.unwrap();
    assert_eq!(check(&app, &raw).await, StatusCode::NO_CONTENT);
    assert_eq!(
        expiry().await.unwrap(),
        before,
        "the check moved the expiry"
    );
}

#[sqlx::test]
async fn a_used_token_fails_the_check(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let raw = reset_token(&app, "spender").await;
    let (status, _) = app
        .post(
            RESET_PASSWORD_ROUTE,
            json!({ "token": raw, "new_password": "Reset456!" }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "reset with the token");

    assert_eq!(check(&app, &raw).await, StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn an_expired_token_fails_the_check(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let raw = reset_token(&app, "latecomer").await;
    sqlx::query("UPDATE password_reset_tokens SET expires_at = NOW() - INTERVAL '1 minute'")
        .execute(&app.pool)
        .await
        .unwrap();

    assert_eq!(check(&app, &raw).await, StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn a_bogus_token_fails_the_check(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    assert_eq!(
        check(&app, "deadbeef-not-a-real-token").await,
        StatusCode::BAD_REQUEST
    );
}

#[sqlx::test]
async fn the_client_reads_a_live_and_a_dead_token(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let raw = reset_token(&app, "roundtrip").await;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let service = app
        .router
        .clone()
        .into_make_service_with_connect_info::<SocketAddr>();
    tokio::spawn(async move { axum::serve(listener, service).await });

    let client = ZwipeClient::new(format!("http://{address}").parse().unwrap());
    client
        .check_reset_token(raw.clone())
        .await
        .expect("a live token checks out");
    let dead = client
        .check_reset_token("deadbeef-not-a-real-token".to_string())
        .await;
    assert!(
        matches!(dead, Err(ClientError::BadRequest(_))),
        "a dead token reads as BadRequest, got {dead:?}"
    );
    client
        .check_reset_token(raw)
        .await
        .expect("the client's check didn't spend the token");
}
