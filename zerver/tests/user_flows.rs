//! User account mutations through the real router: change username / email /
//! password (each re-auths on the current password), and delete-account cascade.
//! Each test uses its own user because the sensitive-route governor (burst 2,
//! then 1/30min, keyed by user id) would 429 a third mutation for one user.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use zwipe_core::http::paths::{
    CHANGE_EMAIL_ROUTE, CHANGE_PASSWORD_ROUTE, CHANGE_USERNAME_ROUTE, CONFIRM_EMAIL_CHANGE_ROUTE,
    DECK_ROUTE, DELETE_USER_ROUTE, GET_USER_ROUTE, LOGIN_ROUTE, PREFERENCES_ROUTE,
    RESEND_VERIFICATION_ROUTE,
};
mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::json;

#[sqlx::test]
async fn change_username_updates_profile(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (token, _) = app.register("olduser").await;

    let (status, updated) = app
        .patch(
            CHANGE_USERNAME_ROUTE,
            json!({ "new_username": "newuser", "password": "TestPass123!" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "change username: {updated}");
    assert_eq!(updated["username"], "newuser");

    let (_, me) = app.get(GET_USER_ROUTE, Some(&token)).await;
    assert_eq!(
        me["username"], "newuser",
        "GET /api/user reflects the new name"
    );
}

#[sqlx::test]
async fn change_username_wrong_password_rejected(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (token, _) = app.register("stayput").await;

    let (status, _) = app
        .patch(
            CHANGE_USERNAME_ROUTE,
            json!({ "new_username": "hacker", "password": "WrongPass123!" }),
            Some(&token),
        )
        .await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "wrong password must not change the username"
    );

    let (_, me) = app.get(GET_USER_ROUTE, Some(&token)).await;
    assert_eq!(me["username"], "stayput");
}

#[sqlx::test]
async fn change_password_then_login_with_new(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (token, _) = app.register("pwuser").await;

    let (status, _) = app
        .patch(
            CHANGE_PASSWORD_ROUTE,
            json!({ "current_password": "TestPass123!", "new_password": "NewPass456!" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "change password");

    // the new password authenticates
    let (status, _) = app
        .post(
            LOGIN_ROUTE,
            json!({ "identifier": "pwuser", "password": "NewPass456!" }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "login with new password");

    // the old one no longer does
    let (status, _) = app
        .post(
            LOGIN_ROUTE,
            json!({ "identifier": "pwuser", "password": "TestPass123!" }),
            None,
        )
        .await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "old password must be rejected"
    );
}

/// Asks for `email`.
async fn request_change(
    app: &TestApp,
    token: &str,
    email: &str,
) -> (StatusCode, serde_json::Value) {
    app.patch(
        CHANGE_EMAIL_ROUTE,
        json!({ "email": email, "password": "TestPass123!" }),
        Some(token),
    )
    .await
}

/// Lets a background send that was going to happen land, so a test can say
/// none did.
async fn settle() {
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
}

#[sqlx::test]
async fn change_email_answers_the_same_for_a_taken_address(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let _ = app.register("owner").await;
    let (token, _) = app.register("prober").await;

    let (taken_status, taken) = request_change(&app, &token, "owner@test.local").await;
    let (free_status, free) = request_change(&app, &token, "fresh@test.local").await;

    assert_eq!(taken_status, StatusCode::OK, "taken: {taken}");
    assert_eq!(free_status, StatusCode::OK, "free: {free}");
    for body in [&taken, &free] {
        assert_eq!(
            body["email"], "prober@test.local",
            "email unchanged: {body}"
        );
    }
    assert_eq!(taken["pending_email"], "owner@test.local");
    assert_eq!(free["pending_email"], "fresh@test.local");

    let keys = |v: &serde_json::Value| {
        let mut k: Vec<String> = v.as_object().unwrap().keys().cloned().collect();
        k.sort();
        k
    };
    assert_eq!(keys(&taken), keys(&free), "same user JSON shape");
    assert_eq!(taken["email_verified_at"], free["email_verified_at"]);
}

/// 1.13.x still sends the retired `confirm_first` flag; it is ignored.
#[sqlx::test]
async fn change_email_ignores_the_retired_flag(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (token, _) = app.register("flagged").await;

    let (status, body) = app
        .patch(
            CHANGE_EMAIL_ROUTE,
            json!({ "email": "moved@test.local", "password": "TestPass123!", "confirm_first": true }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["email"], "flagged@test.local");
    assert_eq!(body["pending_email"], "moved@test.local");
}

#[sqlx::test]
async fn change_email_free_address_switches_on_confirm(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (token, _) = app.register("mover").await;
    let before = app.emails.count();

    let (status, body) = request_change(&app, &token, "moved@test.local").await;
    assert_eq!(status, StatusCode::OK, "request: {body}");
    app.emails.wait_for(before + 1).await;
    assert_eq!(app.emails.last_to().as_deref(), Some("moved@test.local"));
    let raw = app.emails.last_token("confirm-email").unwrap();

    let (_, me) = app.get(GET_USER_ROUTE, Some(&token)).await;
    assert_eq!(
        me["email"], "mover@test.local",
        "not switched before confirm"
    );
    assert_eq!(
        me["pending_email"], "moved@test.local",
        "pending shows: {me}"
    );

    let (status, _) = app
        .post(CONFIRM_EMAIL_CHANGE_ROUTE, json!({ "token": raw }), None)
        .await;
    assert_eq!(status, StatusCode::OK, "confirm");

    let (_, me) = app.get(GET_USER_ROUTE, Some(&token)).await;
    assert_eq!(me["email"], "moved@test.local");
    assert!(
        me["email_verified_at"].is_string(),
        "verified by the click: {me}"
    );
    assert!(me.get("pending_email").is_none(), "pending cleared: {me}");

    assert_eq!(
        app.emails.last_to().as_deref(),
        Some("mover@test.local"),
        "the old address is told"
    );
    assert!(app.emails.last_body().unwrap().contains("moved@test.local"));

    let (status, _) = app
        .post(CONFIRM_EMAIL_CHANGE_ROUTE, json!({ "token": raw }), None)
        .await;
    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "a link works once"
    );
}

#[sqlx::test]
async fn change_email_taken_address_sends_nothing(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let _ = app.register("holder").await;
    let (token, uid) = app.register("claimant").await;
    let before = app.emails.count();

    let (status, _) = request_change(&app, &token, "holder@test.local").await;
    assert_eq!(status, StatusCode::OK);
    settle().await;
    assert_eq!(app.emails.count(), before, "no email to anyone");

    let owners: i64 =
        sqlx::query_scalar("SELECT count(*) FROM users WHERE email = 'holder@test.local'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(owners, 1, "the owner keeps the address");
    let mine: String = sqlx::query_scalar("SELECT email FROM users WHERE id = $1::uuid")
        .bind(&uid)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(mine, "claimant@test.local");
}

#[sqlx::test]
async fn change_email_new_request_replaces_the_old(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (token, _) = app.register("fickle").await;

    let before = app.emails.count();
    let _ = request_change(&app, &token, "first@test.local").await;
    app.emails.wait_for(before + 1).await;
    let first = app.emails.last_token("confirm-email").unwrap();

    let _ = request_change(&app, &token, "second@test.local").await;
    app.emails.wait_for(before + 2).await;
    let second = app.emails.last_token("confirm-email").unwrap();

    let (status, _) = app
        .post(CONFIRM_EMAIL_CHANGE_ROUTE, json!({ "token": first }), None)
        .await;
    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "the first link is dead"
    );

    let (status, _) = app
        .post(CONFIRM_EMAIL_CHANGE_ROUTE, json!({ "token": second }), None)
        .await;
    assert_eq!(status, StatusCode::OK);
    let (_, me) = app.get(GET_USER_ROUTE, Some(&token)).await;
    assert_eq!(me["email"], "second@test.local");
}

#[sqlx::test]
async fn change_email_fails_when_the_address_was_taken_meanwhile(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (token, _) = app.register("slowpoke").await;

    let before = app.emails.count();
    let _ = request_change(&app, &token, "late@test.local").await;
    app.emails.wait_for(before + 1).await;
    let raw = app.emails.last_token("confirm-email").unwrap();

    // someone registers the address before the link is clicked
    let _ = app.register("late").await;

    let (status, body) = app
        .post(CONFIRM_EMAIL_CHANGE_ROUTE, json!({ "token": raw }), None)
        .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");

    let (_, me) = app.get(GET_USER_ROUTE, Some(&token)).await;
    assert_eq!(me["email"], "slowpoke@test.local");
    assert!(
        me.get("pending_email").is_none(),
        "the request is spent: {me}"
    );

    let (status, _) = app
        .post(CONFIRM_EMAIL_CHANGE_ROUTE, json!({ "token": raw }), None)
        .await;
    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "and the link is dead"
    );
}

#[sqlx::test]
async fn resend_verification_leaves_a_pending_change_alone(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (token, _) = app.register("juggler").await;

    let before = app.emails.count();
    let _ = request_change(&app, &token, "juggled@test.local").await;
    app.emails.wait_for(before + 1).await;
    let raw = app.emails.last_token("confirm-email").unwrap();

    let (status, body) = app
        .post(RESEND_VERIFICATION_ROUTE, json!({}), Some(&token))
        .await;
    assert_eq!(status, StatusCode::OK, "resend: {body}");

    let (status, _) = app
        .post(CONFIRM_EMAIL_CHANGE_ROUTE, json!({ "token": raw }), None)
        .await;
    assert_eq!(status, StatusCode::OK, "the change link still works");
}

#[sqlx::test]
async fn delete_user_cascades_decks(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, uid) = app.register("goner").await;
    app.verify_email(&uid).await;

    // give the user a deck to prove the FK cascade removes it
    let (status, _) = app
        .post(
            DECK_ROUTE,
            json!({ "name": "Doomed", "format": "commander" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, _) = app
        .delete_json(
            DELETE_USER_ROUTE,
            json!({ "password": "TestPass123!" }),
            Some(&token),
        )
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

#[sqlx::test]
async fn universes_beyond_preference_round_trips(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (token, _) = app.register("ubuser").await;

    // Defaults: off, empty whitelist.
    let (status, prefs) = app.get(PREFERENCES_ROUTE, Some(&token)).await;
    assert_eq!(status, StatusCode::OK, "get preferences: {prefs}");
    assert_eq!(prefs["exclude_universes_beyond"], false);
    assert_eq!(prefs["universes_beyond_exceptions"], json!([]));

    // Set both; the theme fields stay untouched (partial update).
    let (status, updated) = app
        .patch(
            PREFERENCES_ROUTE,
            json!({
                "exclude_universes_beyond": true,
                "universes_beyond_exceptions": ["middle-earth", "final-fantasy"],
            }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "update: {updated}");
    assert_eq!(updated["exclude_universes_beyond"], true);
    assert_eq!(
        updated["universes_beyond_exceptions"],
        json!(["middle-earth", "final-fantasy"])
    );

    // A theme-only update (an old client's payload) must not wipe them.
    let (status, updated) = app
        .patch(
            PREFERENCES_ROUTE,
            json!({ "theme": "dracula", "dark_mode": false }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "theme-only update: {updated}");
    assert_eq!(updated["theme"], "dracula");
    assert_eq!(updated["exclude_universes_beyond"], true);
    assert_eq!(
        updated["universes_beyond_exceptions"],
        json!(["middle-earth", "final-fantasy"])
    );

    // Unknown franchise slugs are rejected.
    let (status, body) = app
        .patch(
            PREFERENCES_ROUTE,
            json!({ "universes_beyond_exceptions": ["not-a-franchise"] }),
            Some(&token),
        )
        .await;
    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "unknown slug: {body}"
    );
}

#[sqlx::test]
async fn visual_studio_moves_to_vs_code(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, _) = app.register("vsuser").await;

    // An old client still sending the removed slug lands on its replacement.
    let (status, updated) = app
        .patch(
            PREFERENCES_ROUTE,
            json!({ "theme": "visual-studio" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "old slug: {updated}");
    assert_eq!(updated["theme"], "vscode");

    // A row stored before the removal moves with the migration.
    sqlx::query("UPDATE user_preferences SET theme = 'visual-studio'")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::raw_sql(include_str!(
        "../migrations/20261021200000_remove_visual_studio_theme.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();
    let (status, prefs) = app.get(PREFERENCES_ROUTE, Some(&token)).await;
    assert_eq!(status, StatusCode::OK, "get preferences: {prefs}");
    assert_eq!(prefs["theme"], "vscode");
}
