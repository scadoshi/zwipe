//! Auth edge cases that unit tests can't reach: the email-token round-trips
//! (verify-email, password-reset) driven through the captured `FakeEmailSender`
//! exactly as a user would from their inbox, refresh-token single-use rotation,
//! and the login rate-limit lockout (5 / 6s per IP).
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::json;

#[sqlx::test]
async fn verify_email_via_captured_token(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (token, _uid) = app.register("verifier").await;

    // Registration sends the verification email; pull the raw token out of it.
    let raw = app.emails.last_token("verify").expect("verification email with a /verify/ link");

    let (status, _) = app.post("/api/auth/verify-email", json!({ "token": raw }), None).await;
    assert_eq!(status, StatusCode::OK, "verify-email with the captured token");

    // the account now reads as verified
    let (_, me) = app.get("/api/user", Some(&token)).await;
    assert!(me["email_verified_at"].is_string(), "email_verified_at set: {me}");
}

#[sqlx::test]
async fn verify_email_rejects_garbage_token(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let _ = app.register("skeptic").await;
    let (status, _) = app
        .post("/api/auth/verify-email", json!({ "token": "deadbeef-not-a-real-token" }), None)
        .await;
    assert_ne!(status, StatusCode::OK, "an invalid token must not verify");
}

#[sqlx::test]
async fn password_reset_via_captured_token(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let _ = app.register("forgetful").await;

    // request the reset, then read the raw token from the captured email
    let (status, _) = app
        .post("/api/auth/forgot-password", json!({ "email": "forgetful@test.local" }), None)
        .await;
    assert_eq!(status, StatusCode::OK, "forgot-password accepted");
    let raw = app.emails.last_token("reset").expect("reset email with a /reset/ link");

    let (status, _) = app
        .post(
            "/api/auth/reset-password",
            json!({ "token": raw, "new_password": "Reset456!" }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "reset-password with the captured token");

    // the new password authenticates; the old one does not
    let (status, _) = app
        .post("/api/auth/login", json!({ "identifier": "forgetful", "password": "Reset456!" }), None)
        .await;
    assert_eq!(status, StatusCode::OK, "login with the reset password");
    let (status, _) = app
        .post("/api/auth/login", json!({ "identifier": "forgetful", "password": "TestPass123!" }), None)
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "old password rejected after reset");
}

#[sqlx::test]
async fn refresh_token_is_single_use(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let _ = app.register("roller").await;

    let (status, session) = app
        .post("/api/auth/login", json!({ "identifier": "roller", "password": "TestPass123!" }), None)
        .await;
    assert_eq!(status, StatusCode::OK, "login: {session}");
    let user_id = session["user"]["id"].as_str().unwrap().to_string();
    let refresh = session["refresh_token"]["value"].as_str().unwrap().to_string();

    // first use rotates the token and succeeds
    let (status, rotated) = app
        .post(
            "/api/auth/refresh",
            json!({ "user_id": user_id, "refresh_token": refresh }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "first refresh: {rotated}");

    // reusing the now-rotated token must fail
    let (status, _) = app
        .post(
            "/api/auth/refresh",
            json!({ "user_id": user_id, "refresh_token": refresh }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "a rotated refresh token is single-use");
}

#[sqlx::test]
async fn login_rate_limit_locks_out(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let _ = app.register("target").await;

    // The login limiter is burst 5 per IP; all requests here share one fake IP,
    // so rapid attempts exhaust it. Wrong-password attempts 401 until the
    // governor cuts in with 429.
    let mut saw_429 = false;
    for _ in 0..8 {
        let (status, _) = app
            .post("/api/auth/login", json!({ "identifier": "target", "password": "WrongPass123!" }), None)
            .await;
        if status == StatusCode::TOO_MANY_REQUESTS {
            saw_429 = true;
            break;
        }
        assert_eq!(status, StatusCode::UNAUTHORIZED, "pre-limit attempts are 401");
    }
    assert!(saw_429, "the login limiter should 429 after the burst is spent");

    // once limited, even the correct credentials are refused (429, not 200)
    let (status, _) = app
        .post("/api/auth/login", json!({ "identifier": "target", "password": "TestPass123!" }), None)
        .await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS, "correct creds stay locked out while limited");
}
