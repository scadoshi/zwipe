//! A password over the server's verify cap (`MAX_VERIFIED_LENGTH`, 1,024
//! characters) is turned away with a 422 ("password too long") before any
//! lookup or Argon2, on login and on every current-password check, where a
//! wrong password stays a 401.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::json;
use zwipe_core::http::paths::{CHANGE_PASSWORD_ROUTE, CHANGE_USERNAME_ROUTE, LOGIN_ROUTE};

/// One character over the cap.
fn too_long() -> String {
    "a".repeat(1_025)
}

#[sqlx::test]
async fn login_turns_away_an_oversized_password(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    app.register("capped").await;

    let (status, body) = app
        .post(
            LOGIN_ROUTE,
            json!({ "identifier": "capped", "password": too_long() }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");

    let (status, body) = app
        .post(
            LOGIN_ROUTE,
            json!({ "identifier": "nobody-here", "password": too_long() }),
            None,
        )
        .await;
    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "the same answer for an unknown account: {body}"
    );

    let (status, _) = app
        .post(
            LOGIN_ROUTE,
            json!({ "identifier": "capped", "password": "Wrong123!" }),
            None,
        )
        .await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "a wrong password is unchanged"
    );

    let (status, _) = app
        .post(
            LOGIN_ROUTE,
            json!({ "identifier": "capped", "password": "TestPass123!" }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "the real password still signs in");
}

#[sqlx::test]
async fn a_current_password_check_turns_away_an_oversized_password(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (token, _) = app.register("recheck").await;

    let (status, body) = app
        .patch(
            CHANGE_PASSWORD_ROUTE,
            json!({ "current_password": too_long(), "new_password": "NewPass456!" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");

    let (status, body) = app
        .patch(
            CHANGE_USERNAME_ROUTE,
            json!({ "new_username": "recheck2", "password": too_long() }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
}
