//! User account mutations through the real router: change username / email /
//! password (each re-auths on the current password), and delete-account cascade.
//! Each test uses its own user because the sensitive-route governor (burst 2,
//! then 1/30min, keyed by user id) would 429 a third mutation for one user.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::json;

#[sqlx::test]
async fn change_username_updates_profile(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (token, _) = app.register("olduser").await;

    let (status, updated) = app
        .put(
            "/api/user/change-username",
            json!({ "new_username": "newuser", "password": "TestPass123!" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "change username: {updated}");
    assert_eq!(updated["username"], "newuser");

    let (_, me) = app.get("/api/user", Some(&token)).await;
    assert_eq!(me["username"], "newuser", "GET /api/user reflects the new name");
}

#[sqlx::test]
async fn change_username_wrong_password_rejected(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (token, _) = app.register("stayput").await;

    let (status, _) = app
        .put(
            "/api/user/change-username",
            json!({ "new_username": "hacker", "password": "WrongPass123!" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "wrong password must not change the username");

    let (_, me) = app.get("/api/user", Some(&token)).await;
    assert_eq!(me["username"], "stayput");
}

#[sqlx::test]
async fn change_password_then_login_with_new(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (token, _) = app.register("pwuser").await;

    let (status, _) = app
        .put(
            "/api/user/change-password",
            json!({ "current_password": "TestPass123!", "new_password": "NewPass456!" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "change password");

    // the new password authenticates
    let (status, _) = app
        .post("/api/auth/login", json!({ "identifier": "pwuser", "password": "NewPass456!" }), None)
        .await;
    assert_eq!(status, StatusCode::OK, "login with new password");

    // the old one no longer does
    let (status, _) = app
        .post("/api/auth/login", json!({ "identifier": "pwuser", "password": "TestPass123!" }), None)
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "old password must be rejected");
}

#[sqlx::test]
async fn change_email_updates_profile(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (token, _) = app.register("mailer").await;

    let (status, updated) = app
        .put(
            "/api/user/change-email",
            json!({ "email": "moved@test.local", "password": "TestPass123!" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "change email: {updated}");
    assert_eq!(updated["email"], "moved@test.local");
}

#[sqlx::test]
async fn delete_user_cascades_decks(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, uid) = app.register("goner").await;
    app.verify_email(&uid).await;

    // give the user a deck to prove the FK cascade removes it
    let (status, _) = app
        .post("/api/deck", json!({ "name": "Doomed", "format": "commander" }), Some(&token))
        .await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, _) = app
        .delete_json("/api/user/delete-user", json!({ "password": "TestPass123!" }), Some(&token))
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT, "delete account");

    let user_uuid: uuid::Uuid = uid.parse().unwrap();
    let users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE id = $1")
        .bind(user_uuid)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(users, 0, "user row gone");
    let decks: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM decks WHERE user_id = $1")
        .bind(user_uuid)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(decks, 0, "decks cascaded on user delete");
}
