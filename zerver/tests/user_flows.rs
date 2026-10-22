//! User account mutations through the real router: change username / email /
//! password (each re-auths on the current password), and delete-account cascade.
//! Each test uses its own user because the sensitive-route governor (burst 2,
//! then 1/30min, keyed by user id) would 429 a third mutation for one user.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use zwipe_core::http::paths::{
    CHANGE_EMAIL_ROUTE, CHANGE_PASSWORD_ROUTE, CHANGE_USERNAME_ROUTE, DECK_ROUTE,
    DELETE_USER_ROUTE, GET_USER_ROUTE, LOGIN_ROUTE, PREFERENCES_ROUTE,
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

#[sqlx::test]
async fn change_email_updates_profile(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (token, _) = app.register("mailer").await;

    let (status, updated) = app
        .patch(
            CHANGE_EMAIL_ROUTE,
            json!({ "email": "moved@test.local", "password": "TestPass123!" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "change email: {updated}");
    assert_eq!(updated["email"], "moved@test.local");
}

/// A new address that belongs to another account must look exactly like a
/// successful change from the outside, so the endpoint cannot be used to
/// learn which addresses have accounts. The real owner is told instead.
#[sqlx::test]
async fn change_email_to_taken_address_answers_like_success_and_tells_owner(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (_, _) = app.register("owner").await;
    let (token, claimant_id) = app.register("claimant").await;

    // a real change, for the shape to compare against
    let (ok_status, changed) = app
        .patch(
            CHANGE_EMAIL_ROUTE,
            json!({ "email": "claimant2@test.local", "password": "TestPass123!" }),
            Some(&token),
        )
        .await;
    assert_eq!(ok_status, StatusCode::OK, "change email: {changed}");

    // (to, subject, body) snapshots; `SendEmail` is not `Clone`
    let snapshot = || -> Vec<(String, String, String)> {
        app.emails
            .sent
            .lock()
            .unwrap()
            .iter()
            .map(|e| (e.to.clone(), e.subject.clone(), e.html_body.clone()))
            .collect()
    };
    let sent_before = snapshot();
    let to_owner_before = sent_before
        .iter()
        .filter(|(to, _, _)| to == "owner@test.local")
        .count();

    let (status, body) = app
        .patch(
            CHANGE_EMAIL_ROUTE,
            json!({ "email": "owner@test.local", "password": "TestPass123!" }),
            Some(&token),
        )
        .await;
    assert_eq!(
        status, ok_status,
        "taken address must answer like success: {body}"
    );
    let keys = |v: &serde_json::Value| {
        let mut k: Vec<String> = v.as_object().unwrap().keys().cloned().collect();
        k.sort();
        k
    };
    assert_eq!(keys(&body), keys(&changed), "same user JSON shape");
    assert_eq!(body["id"], changed["id"]);
    assert_eq!(
        body["email"], "claimant2@test.local",
        "caller's email unchanged in response"
    );

    let claimant_uuid: uuid::Uuid = claimant_id.parse().unwrap();
    let stored: String = sqlx::query_scalar("SELECT email FROM users WHERE id = $1")
        .bind(claimant_uuid)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        stored, "claimant2@test.local",
        "caller's email unchanged in the database"
    );

    let sent_after = snapshot();
    assert_eq!(
        sent_after.len(),
        sent_before.len() + 1,
        "the duplicate attempt sends exactly one email"
    );
    let to_owner: Vec<_> = sent_after
        .iter()
        .filter(|(to, _, _)| to == "owner@test.local")
        .collect();
    assert_eq!(
        to_owner.len(),
        to_owner_before + 1,
        "the owner got exactly one more email"
    );
    let (_, subject, html_body) = to_owner.last().unwrap();
    assert_eq!(subject, "Someone tried to use your email on Zwipe");
    assert!(html_body.contains("Nothing changed"), "{html_body}");
    assert!(
        !html_body.contains("claimant"),
        "the owner's email must not name the other account"
    );
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
