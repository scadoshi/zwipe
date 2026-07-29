//! Auth edge cases that unit tests can't reach: the email-token round-trips
//! (verify-email, password-reset) driven through the captured `FakeEmailSender`
//! exactly as a user would from their inbox, refresh-token single-use rotation,
//! the login rate-limit lockout (5 / 6s per IP), and the insert-time refresh-token
//! prune (expired rows purged + session cap enforced, no global sweeper).
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
    let raw = app
        .emails
        .last_token("verify")
        .expect("verification email with a /verify/ link");

    let (status, _) = app
        .post("/api/auth/verify-email", json!({ "token": raw }), None)
        .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "verify-email with the captured token"
    );

    // the account now reads as verified
    let (_, me) = app.get("/api/user", Some(&token)).await;
    assert!(
        me["email_verified_at"].is_string(),
        "email_verified_at set: {me}"
    );
}

#[sqlx::test]
async fn verify_email_rejects_garbage_token(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let _ = app.register("skeptic").await;
    let (status, _) = app
        .post(
            "/api/auth/verify-email",
            json!({ "token": "deadbeef-not-a-real-token" }),
            None,
        )
        .await;
    assert_ne!(status, StatusCode::OK, "an invalid token must not verify");
}

#[sqlx::test]
async fn password_reset_via_captured_token(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let _ = app.register("forgetful").await;

    // request the reset, then read the raw token from the captured email
    let (status, _) = app
        .post(
            "/api/auth/forgot-password",
            json!({ "email": "forgetful@test.local" }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "forgot-password accepted");
    let raw = app
        .emails
        .last_token("reset")
        .expect("reset email with a /reset/ link");

    let (status, _) = app
        .post(
            "/api/auth/reset-password",
            json!({ "token": raw, "new_password": "Reset456!" }),
            None,
        )
        .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "reset-password with the captured token"
    );

    // the new password authenticates; the old one does not
    let (status, _) = app
        .post(
            "/api/auth/login",
            json!({ "identifier": "forgetful", "password": "Reset456!" }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "login with the reset password");
    let (status, _) = app
        .post(
            "/api/auth/login",
            json!({ "identifier": "forgetful", "password": "TestPass123!" }),
            None,
        )
        .await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "old password rejected after reset"
    );
}

#[sqlx::test]
async fn refresh_token_is_single_use(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let _ = app.register("roller").await;

    let (status, session) = app
        .post(
            "/api/auth/login",
            json!({ "identifier": "roller", "password": "TestPass123!" }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "login: {session}");
    let user_id = session["user"]["id"].as_str().unwrap().to_string();
    let refresh = session["refresh_token"]["value"]
        .as_str()
        .unwrap()
        .to_string();

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
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "a rotated refresh token is single-use"
    );
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
            .post(
                "/api/auth/login",
                json!({ "identifier": "target", "password": "WrongPass123!" }),
                None,
            )
            .await;
        if status == StatusCode::TOO_MANY_REQUESTS {
            saw_429 = true;
            break;
        }
        assert_eq!(
            status,
            StatusCode::UNAUTHORIZED,
            "pre-limit attempts are 401"
        );
    }
    assert!(
        saw_429,
        "the login limiter should 429 after the burst is spent"
    );

    // once limited, even the correct credentials are refused (429, not 200)
    let (status, _) = app
        .post(
            "/api/auth/login",
            json!({ "identifier": "target", "password": "TestPass123!" }),
            None,
        )
        .await;
    assert_eq!(
        status,
        StatusCode::TOO_MANY_REQUESTS,
        "correct creds stay locked out while limited"
    );
}

#[sqlx::test]
async fn session_insert_prunes_expired_and_caps(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (_, uid) = app.register("hoarder").await;
    let user_id = uuid::Uuid::parse_str(&uid).unwrap();

    // Plant 3 expired tokens (a dormant user's leftovers) and 6 live ones,
    // staggered so the prune's newest-first ordering is deterministic.
    for i in 0..3i32 {
        sqlx::query(
            "INSERT INTO refresh_tokens (user_id, value_hash, expires_at, created_at)
             VALUES ($1, $2, NOW() - INTERVAL '1 day', NOW() - make_interval(days => 15, mins => $3))",
        )
        .bind(user_id)
        .bind(format!("expired-{i}"))
        .bind(i)
        .execute(&pool)
        .await
        .unwrap();
    }
    for i in 0..6i32 {
        sqlx::query(
            "INSERT INTO refresh_tokens (user_id, value_hash, expires_at, created_at)
             VALUES ($1, $2, NOW() + INTERVAL '14 days', NOW() - make_interval(mins => $3))",
        )
        .bind(user_id)
        .bind(format!("live-{i}"))
        .bind(i + 1)
        .execute(&pool)
        .await
        .unwrap();
    }

    // One login triggers the insert-time prune (no global sweeper involved).
    let (status, _) = app
        .post(
            "/api/auth/login",
            json!({ "identifier": "hoarder", "password": "TestPass123!" }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "login");

    let expired: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM refresh_tokens WHERE user_id = $1 AND expires_at < NOW()",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(expired, 0, "expired tokens purged at insert");

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM refresh_tokens WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(total, 5, "live tokens capped at the session maximum");

    // Newest-first survivors: the login + register rows and live-0..2; the
    // three oldest planted live tokens are the cap's casualties.
    let casualties: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM refresh_tokens WHERE user_id = $1 AND value_hash IN ('live-3', 'live-4', 'live-5')",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(casualties, 0, "oldest live tokens were pruned first");
    let newest: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM refresh_tokens WHERE user_id = $1 AND value_hash = 'live-0'",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(newest, 1, "most recent live token survives");
}
