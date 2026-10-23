//! Registration through the real router: no session, and the same answer
//! whether or not the email already has an account.
//!
//! Requires `DATABASE_URL` (dev's value works): `set -a; source zerver/.env`.

#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use zwipe_core::http::paths::{LOGIN_ROUTE, REGISTER_ROUTE, VERIFY_EMAIL_ROUTE};
mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::{Value, json};

async fn register(app: &TestApp, username: &str, email: &str) -> (StatusCode, Value) {
    app.post(
        REGISTER_ROUTE,
        json!({
            "username": username,
            "email": email,
            "password": "TestPass123!",
        }),
        None,
    )
    .await
}

/// Lets a background send that was going to happen land, so a test can say
/// none did.
async fn settle() {
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
}

#[sqlx::test]
async fn new_email_is_202_without_session_and_can_log_in(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);

    let (status, body) = register(&app, "newbie", "newbie@test.local").await;
    assert_eq!(status, StatusCode::ACCEPTED, "{body}");
    assert_eq!(body, json!({ "status": "verification_sent" }));

    app.emails.wait_for(1).await;
    assert_eq!(app.emails.last_to().as_deref(), Some("newbie@test.local"));
    let raw = app.emails.last_token("verify").unwrap();

    let (status, session) = app
        .post(
            LOGIN_ROUTE,
            json!({ "identifier": "newbie", "password": "TestPass123!" }),
            None,
        )
        .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "unverified accounts can log in: {session}"
    );
    assert!(session["user"]["email_verified_at"].is_null());

    let (status, _) = app
        .post(VERIFY_EMAIL_ROUTE, json!({ "token": raw }), None)
        .await;
    assert_eq!(status, StatusCode::OK);
}

#[sqlx::test]
async fn taken_email_answers_like_a_new_one_and_tells_the_owner(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let _ = app.register("alice").await;
    let users_before: i64 = sqlx::query_scalar("SELECT count(*) FROM users")
        .fetch_one(&pool)
        .await
        .unwrap();
    let emails_before = app.emails.count();

    let (status, taken) = register(&app, "mallory", "alice@test.local").await;
    let (_, fresh) = register(&app, "fresh", "fresh@test.local").await;
    assert_eq!(status, StatusCode::ACCEPTED, "{taken}");
    assert_eq!(taken, fresh, "byte-equal bodies");

    app.emails.wait_for(emails_before + 2).await;
    let (subject, body) = {
        let sent = app.emails.sent.lock().unwrap();
        let notice = sent
            .iter()
            .rev()
            .find(|e| e.to == "alice@test.local")
            .unwrap();
        (notice.subject.clone(), notice.html_body.clone())
    };
    assert_eq!(subject, "You already have a Zwipe account");
    assert!(!body.contains("/verify/"), "no verify link for the owner");

    let users_after: i64 = sqlx::query_scalar("SELECT count(*) FROM users")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(users_after, users_before + 1, "only fresh was created");

    let (status, _) = app
        .post(
            LOGIN_ROUTE,
            json!({ "identifier": "alice", "password": "TestPass123!" }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "alice's account is untouched");
}

#[sqlx::test]
async fn taken_username_is_409(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let _ = app.register("bob").await;

    let (status, _) = register(&app, "bob", "other@test.local").await;
    assert_eq!(status, StatusCode::CONFLICT);
}

/// What 1.13.x sends: the retired `verify_first` flag and `client_version`
/// are ignored, not rejected.
#[sqlx::test]
async fn retired_fields_are_ignored(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (status, body) = app
        .post(
            REGISTER_ROUTE,
            json!({
                "username": "oldapp",
                "email": "oldapp@test.local",
                "password": "TestPass123!",
                "client_version": "1.13.0",
                "verify_first": true,
            }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::ACCEPTED, "{body}");
    assert_eq!(body, json!({ "status": "verification_sent" }));
}

/// A client that leaves the flag out gets the same flat answer: the old
/// auto-login path is gone.
#[sqlx::test]
async fn no_flag_no_session(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let _ = app.register("dave").await;
    let (status, body) = register(&app, "dave2", "dave@test.local").await;
    assert_eq!(status, StatusCode::ACCEPTED, "{body}");
    assert!(body.get("access_token").is_none());
}

#[sqlx::test]
async fn account_exists_notice_is_on_cooldown(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let _ = app.register("carol").await;
    let before = app.emails.count();

    let _ = register(&app, "copy1", "carol@test.local").await;
    let _ = register(&app, "copy2", "carol@test.local").await;
    app.emails.wait_for(before + 1).await;
    settle().await;
    assert_eq!(app.emails.count(), before + 1, "one notice an hour");
}
