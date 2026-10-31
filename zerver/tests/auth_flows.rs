//! Auth HTTP flows through the real router: register (202, no session) → login
//! → authed request → login again → refresh (token rotation). Proves the whole stack (handler + middleware +
//! service + repo + real SQL) works end-to-end in a test.
//!
//! Requires `DATABASE_URL` (dev's value works): `set -a; source zerver/.env`.

use zwipe_core::http::paths::{GET_USER_ROUTE, LOGIN_ROUTE, REFRESH_SESSION_ROUTE};
mod common;

use axum::http::StatusCode;
use common::TestApp;

#[sqlx::test]
async fn register_authed_login_refresh(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);

    // 1. Register, then log in: the helper does both.
    let (token, user_id) = app.register("alice").await;
    assert_ne!(token, "");

    // 2. Authed request with the access token.
    let (status, me) = app.get(GET_USER_ROUTE, Some(&token)).await;
    assert_eq!(status, StatusCode::OK, "GET /api/user: {me}");
    assert_eq!(me["username"], "alice");

    // 3. Login with the same credentials.
    let (status, session) = app
        .post(
            LOGIN_ROUTE,
            serde_json::json!({ "identifier": "alice", "password": "TestPass123!" }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "login: {session}");
    let refresh = session["refresh_token"]["value"]
        .as_str()
        .unwrap()
        .to_string();

    // 4. Refresh: the old token rotates into a new session.
    let (status, refreshed) = app
        .post(
            REFRESH_SESSION_ROUTE,
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
            LOGIN_ROUTE,
            serde_json::json!({ "identifier": "bob", "password": "WrongPass123!" }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn authed_route_without_token_is_401(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (status, _) = app.get(GET_USER_ROUTE, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

/// Argon2 runs on the blocking pool: two logins at once both succeed, and a
/// ticker on the same task keeps ticking while they hash, which it can't if
/// the hashing holds the runtime thread.
#[sqlx::test]
async fn concurrent_logins_leave_the_runtime_free(pool: sqlx::PgPool) {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::{Duration, Instant};

    let app = TestApp::new(pool);
    app.register("hasher1").await;
    app.register("hasher2").await;

    let done = AtomicBool::new(false);
    let ticker = async {
        let mut longest = Duration::ZERO;
        let mut last = Instant::now();
        while !done.load(Ordering::SeqCst) {
            tokio::time::sleep(Duration::from_millis(1)).await;
            longest = longest.max(last.elapsed());
            last = Instant::now();
        }
        longest
    };
    let logins = async {
        let results = tokio::join!(
            app.post(
                LOGIN_ROUTE,
                serde_json::json!({ "identifier": "hasher1", "password": "TestPass123!" }),
                None,
            ),
            app.post(
                LOGIN_ROUTE,
                serde_json::json!({ "identifier": "hasher2", "password": "TestPass123!" }),
                None,
            ),
        );
        done.store(true, Ordering::SeqCst);
        results
    };
    let (longest, ((first, first_body), (second, second_body))) = tokio::join!(ticker, logins);
    assert_eq!(first, StatusCode::OK, "first login: {first_body}");
    assert_eq!(second, StatusCode::OK, "second login: {second_body}");
    assert!(
        longest < Duration::from_millis(150),
        "the runtime stalled for {longest:?} during the logins"
    );
}

#[sqlx::test]
async fn terminal_platform_survives_login_and_refresh(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (_, user_id) = app.register("tui").await;

    let (status, session) = app
        .post(
            LOGIN_ROUTE,
            serde_json::json!({
                "identifier": "tui",
                "password": "TestPass123!",
                "platform": "terminal",
                "client_version": "0.1.0"
            }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "terminal login: {session}");
    let refresh = session["refresh_token"]["value"]
        .as_str()
        .unwrap()
        .to_string();

    let (status, refreshed) = app
        .post(
            REFRESH_SESSION_ROUTE,
            serde_json::json!({ "user_id": user_id, "refresh_token": refresh }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "refresh: {refreshed}");

    // The rotated row carries the platform forward from the stored string.
    let platforms: Vec<Option<String>> = sqlx::query_scalar(
        "SELECT platform FROM refresh_tokens WHERE replaced_at IS NULL AND platform IS NOT NULL",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(platforms, vec![Some("terminal".to_string())]);
}
