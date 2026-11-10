//! User account mutations through the real router: change username / email /
//! password (each re-auths on the current password), and delete-account cascade.
//! Each test uses its own user because the sensitive-route governor (burst 2,
//! then 1/30min, keyed by user id) would 429 a third mutation for one user.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![expect(
    clippy::arithmetic_side_effects,
    reason = "a test: email and row counts add small numbers, and an overflow would only panic the test"
)]

use zwipe_core::http::paths::{
    CANCEL_EMAIL_CHANGE_ROUTE, CHANGE_EMAIL_ROUTE, CHANGE_PASSWORD_ROUTE, CHANGE_USERNAME_ROUTE,
    CONFIRM_EMAIL_CHANGE_ROUTE, DECK_ROUTE, DELETE_USER_ROUTE, GET_USER_ROUTE, LOGIN_ROUTE,
    MARK_HINT_SHOWN_ROUTE, REFRESH_SESSION_ROUTE, RESEND_EMAIL_CHANGE_ROUTE,
    RESEND_VERIFICATION_ROUTE, SETTINGS_ROUTE,
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
async fn change_username_to_the_current_one_is_refused(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (token, _) = app.register("samename").await;

    let (status, body) = app
        .patch(
            CHANGE_USERNAME_ROUTE,
            json!({ "new_username": "samename", "password": "TestPass123!" }),
            Some(&token),
        )
        .await;
    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "unchanged: {body}"
    );

    // A change of case is a different username.
    let (status, updated) = app
        .patch(
            CHANGE_USERNAME_ROUTE,
            json!({ "new_username": "SameName", "password": "TestPass123!" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "case change: {updated}");
    assert_eq!(updated["username"], "SameName");
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
    assert!(taken["pending_email_expires_at"].is_string(), "{taken}");

    let keys = |v: &serde_json::Value| {
        let mut k: Vec<String> = v.as_object().unwrap().keys().cloned().collect();
        k.sort();
        k
    };
    assert_eq!(keys(&taken), keys(&free), "same user JSON shape");
    assert_eq!(taken["email_verified_at"], free["email_verified_at"]);
}

#[sqlx::test]
async fn change_email_to_the_current_one_is_refused(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (token, _) = app.register("sameemail").await;

    let (status, body) = request_change(&app, &token, "sameemail@test.local").await;
    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "unchanged: {body}"
    );

    let (_, me) = app.get(GET_USER_ROUTE, Some(&token)).await;
    assert!(me["pending_email"].is_null(), "nothing pending: {me}");
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
    assert!(
        me["pending_email_expires_at"].is_string(),
        "expiry shows: {me}"
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
    assert!(
        me.get("pending_email_expires_at").is_none(),
        "expiry cleared: {me}"
    );

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
async fn resend_email_change_mails_a_fresh_link_that_replaces_the_old(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (token, _) = app.register("forgetful").await;

    let before = app.emails.count();
    let _ = request_change(&app, &token, "fresh@test.local").await;
    app.emails.wait_for(before + 1).await;
    let first = app.emails.last_token("confirm-email").unwrap();
    let (_, me) = app.get(GET_USER_ROUTE, Some(&token)).await;
    let first_expiry = me["pending_email_expires_at"].clone();

    let (status, body) = app
        .post(RESEND_EMAIL_CHANGE_ROUTE, json!({}), Some(&token))
        .await;
    assert_eq!(status, StatusCode::OK, "resend: {body}");
    app.emails.wait_for(before + 2).await;
    let second = app.emails.last_token("confirm-email").unwrap();
    assert_ne!(first, second, "a new link");

    let (_, me) = app.get(GET_USER_ROUTE, Some(&token)).await;
    assert_eq!(
        me["pending_email"], "fresh@test.local",
        "the same address waits"
    );
    assert_ne!(
        me["pending_email_expires_at"], first_expiry,
        "with a new expiry"
    );

    let (status, _) = app
        .post(CONFIRM_EMAIL_CHANGE_ROUTE, json!({ "token": first }), None)
        .await;
    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "the old link is dead"
    );
    let (status, _) = app
        .post(CONFIRM_EMAIL_CHANGE_ROUTE, json!({ "token": second }), None)
        .await;
    assert_eq!(status, StatusCode::OK, "the new one confirms");
    let (_, me) = app.get(GET_USER_ROUTE, Some(&token)).await;
    assert_eq!(me["email"], "fresh@test.local");
}

#[sqlx::test]
async fn resend_email_change_without_a_pending_change_is_not_found(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (token, uid) = app.register("settled").await;
    app.verify_email(&uid).await;
    let before = app.emails.count();

    let (status, body) = app
        .post(RESEND_EMAIL_CHANGE_ROUTE, json!({}), Some(&token))
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
    settle().await;
    assert_eq!(app.emails.count(), before, "nothing sent");
}

#[sqlx::test]
async fn resend_email_change_to_a_taken_address_sends_nothing(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let _ = app.register("keeper").await;
    let (token, _) = app.register("hopeful").await;

    let (status, _) = request_change(&app, &token, "keeper@test.local").await;
    assert_eq!(status, StatusCode::OK);
    settle().await;
    let before = app.emails.count();

    let (status, body) = app
        .post(RESEND_EMAIL_CHANGE_ROUTE, json!({}), Some(&token))
        .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "answers as for a free address: {body}"
    );
    settle().await;
    assert_eq!(app.emails.count(), before, "no email to anyone");
}

#[sqlx::test]
async fn resend_email_change_is_held_to_one_a_minute(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (token, _) = app.register("eager").await;
    let _ = request_change(&app, &token, "eager-new@test.local").await;

    let (status, _) = app
        .post(RESEND_EMAIL_CHANGE_ROUTE, json!({}), Some(&token))
        .await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = app
        .post(RESEND_EMAIL_CHANGE_ROUTE, json!({}), Some(&token))
        .await;
    assert_eq!(
        status,
        StatusCode::TOO_MANY_REQUESTS,
        "a second press waits"
    );
}

#[sqlx::test]
async fn cancel_email_change_drops_the_pending_change_and_kills_its_link(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (token, _) = app.register("changeable").await;

    let before = app.emails.count();
    let _ = request_change(&app, &token, "second-thoughts@test.local").await;
    app.emails.wait_for(before + 1).await;
    let link = app.emails.last_token("confirm-email").unwrap();

    let (status, body) = app
        .post(CANCEL_EMAIL_CHANGE_ROUTE, json!({}), Some(&token))
        .await;
    assert_eq!(status, StatusCode::OK, "cancel: {body}");

    let (_, me) = app.get(GET_USER_ROUTE, Some(&token)).await;
    assert!(me["pending_email"].is_null(), "nothing pending: {me}");
    assert_eq!(me["email"], "changeable@test.local", "the address stays");

    let (status, _) = app
        .post(CONFIRM_EMAIL_CHANGE_ROUTE, json!({ "token": link }), None)
        .await;
    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "the cancelled link is dead"
    );
    let (_, me) = app.get(GET_USER_ROUTE, Some(&token)).await;
    assert_eq!(me["email"], "changeable@test.local");
}

#[sqlx::test]
async fn cancel_email_change_without_a_pending_change_is_not_found(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (token, _) = app.register("unchanged").await;

    let (status, body) = app
        .post(CANCEL_EMAIL_CHANGE_ROUTE, json!({}), Some(&token))
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
}

#[sqlx::test]
async fn cancel_email_change_twice_is_not_found_the_second_time(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (token, _) = app.register("decisive").await;
    let (status, _) = request_change(&app, &token, "decisive-new@test.local").await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = app
        .post(CANCEL_EMAIL_CHANGE_ROUTE, json!({}), Some(&token))
        .await;
    assert_eq!(status, StatusCode::OK);
    let (status, body) = app
        .post(CANCEL_EMAIL_CHANGE_ROUTE, json!({}), Some(&token))
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "already gone: {body}");
}

#[sqlx::test]
async fn cancel_email_change_needs_a_session(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);

    let (status, _) = app.post(CANCEL_EMAIL_CHANGE_ROUTE, json!({}), None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
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
async fn universes_beyond_setting_round_trips(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (token, _) = app.register("ubuser").await;

    // Defaults: off, empty whitelist.
    let (status, settings) = app.get(SETTINGS_ROUTE, Some(&token)).await;
    assert_eq!(status, StatusCode::OK, "get settings: {settings}");
    assert_eq!(settings["exclude_universes_beyond"], false);
    assert_eq!(settings["universes_beyond_exceptions"], json!([]));

    // Set both; the theme fields stay untouched (partial update).
    let (status, updated) = app
        .patch(
            SETTINGS_ROUTE,
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
            SETTINGS_ROUTE,
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
            SETTINGS_ROUTE,
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

/// Whether `pip_style` is a value the client floor, 1.18.1, parses: its enum
/// reads each value under either name.
fn floor_clients_parse(pip_style: &serde_json::Value) -> bool {
    matches!(
        pip_style.as_str(),
        Some("theme" | "themed" | "natural" | "classic")
    )
}

#[sqlx::test]
async fn settings_responses_parse_as_the_floor_client_reads_them(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (token, user_id) = app.register("shippeduser").await;
    for sent in ["themed", "classic", "theme", "natural"] {
        let (status, updated) = app
            .patch(SETTINGS_ROUTE, json!({ "pip_style": sent }), Some(&token))
            .await;
        assert_eq!(status, StatusCode::OK, "patch {sent}: {updated}");
        assert!(
            floor_clients_parse(&updated["pip_style"]),
            "patch {sent}: {updated}"
        );

        let (status, settings) = app.get(SETTINGS_ROUTE, Some(&token)).await;
        assert_eq!(status, StatusCode::OK, "get after {sent}: {settings}");
        assert!(
            floor_clients_parse(&settings["pip_style"]),
            "get after {sent}: {settings}"
        );

        let (status, session) = app
            .post(
                LOGIN_ROUTE,
                json!({ "identifier": "shippeduser", "password": "TestPass123!" }),
                None,
            )
            .await;
        assert_eq!(status, StatusCode::OK, "login after {sent}: {session}");
        assert!(
            floor_clients_parse(&session["settings"]["pip_style"]),
            "login after {sent}: {session}"
        );

        let refresh = session["refresh_token"]["value"]
            .as_str()
            .unwrap()
            .to_string();
        let (status, refreshed) = app
            .post(
                REFRESH_SESSION_ROUTE,
                json!({ "user_id": user_id, "refresh_token": refresh }),
                None,
            )
            .await;
        assert_eq!(status, StatusCode::OK, "refresh after {sent}: {refreshed}");
        assert!(
            floor_clients_parse(&refreshed["settings"]["pip_style"]),
            "refresh after {sent}: {refreshed}"
        );
    }
}

#[sqlx::test]
async fn swipe_confirmations_setting_round_trips(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, _) = app.register("swipeuser").await;

    // Default: the chip.
    let (status, settings) = app.get(SETTINGS_ROUTE, Some(&token)).await;
    assert_eq!(status, StatusCode::OK, "get settings: {settings}");
    assert_eq!(settings["swipe_confirmations"], "chip");

    for value in ["toasts", "off", "chip", "toasts"] {
        let (status, updated) = app
            .patch(
                SETTINGS_ROUTE,
                json!({ "swipe_confirmations": value }),
                Some(&token),
            )
            .await;
        assert_eq!(status, StatusCode::OK, "update {value}: {updated}");
        assert_eq!(updated["swipe_confirmations"], value);
    }

    // A payload from a client before the field leaves it alone.
    let (status, updated) = app
        .patch(
            SETTINGS_ROUTE,
            json!({ "theme": "dracula", "dark_mode": false, "pip_style": "classic" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "older payload: {updated}");
    assert_eq!(updated["swipe_confirmations"], "toasts");

    let (status, settings) = app.get(SETTINGS_ROUTE, Some(&token)).await;
    assert_eq!(status, StatusCode::OK, "get settings: {settings}");
    assert_eq!(settings["swipe_confirmations"], "toasts");
    let stored: String = sqlx::query_scalar("SELECT swipe_confirmations FROM user_settings")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(stored, "toasts");

    // A value that does not exist is rejected.
    let (status, body) = app
        .patch(
            SETTINGS_ROUTE,
            json!({ "swipe_confirmations": "bogus" }),
            Some(&token),
        )
        .await;
    assert!(status.is_client_error(), "unknown value: {status} {body}");
}

#[sqlx::test]
async fn pip_style_setting_round_trips(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, _) = app.register("pipuser").await;

    // Default: the theme's colors.
    let (status, settings) = app.get(SETTINGS_ROUTE, Some(&token)).await;
    assert_eq!(status, StatusCode::OK, "get settings: {settings}");
    assert_eq!(settings["pip_style"], "themed");

    let (status, updated) = app
        .patch(
            SETTINGS_ROUTE,
            json!({ "pip_style": "classic" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "update: {updated}");
    assert_eq!(updated["pip_style"], "classic");

    // An old client's payload, without the field, leaves it alone.
    let (status, updated) = app
        .patch(
            SETTINGS_ROUTE,
            json!({ "theme": "dracula", "dark_mode": false }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "theme-only update: {updated}");
    assert_eq!(updated["pip_style"], "classic");

    let (status, settings) = app.get(SETTINGS_ROUTE, Some(&token)).await;
    assert_eq!(status, StatusCode::OK, "get settings: {settings}");
    assert_eq!(settings["pip_style"], "classic");

    // The stored row and the wire both say classic.
    let stored: String = sqlx::query_scalar("SELECT pip_style FROM user_settings")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(stored, "classic");

    // `themed` lands, stores as `themed` and answers `themed`.
    let (status, updated) = app
        .patch(
            SETTINGS_ROUTE,
            json!({ "pip_style": "themed" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "reset: {updated}");
    assert_eq!(updated["pip_style"], "themed");
    let stored: String = sqlx::query_scalar("SELECT pip_style FROM user_settings")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(stored, "themed");

    // Builds through 1.18.16 send the earlier names. They land and answer
    // with the current ones, which those builds also read.
    for (sent, current) in [("natural", "classic"), ("theme", "themed")] {
        let (status, updated) = app
            .patch(SETTINGS_ROUTE, json!({ "pip_style": sent }), Some(&token))
            .await;
        assert_eq!(status, StatusCode::OK, "earlier {sent}: {updated}");
        assert_eq!(updated["pip_style"], current, "earlier {sent}");
        let stored: String = sqlx::query_scalar("SELECT pip_style FROM user_settings")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(stored, current, "earlier {sent} stores the current name");
    }

    // A color that does not exist is rejected.
    let (status, body) = app
        .patch(
            SETTINGS_ROUTE,
            json!({ "pip_style": "bogus" }),
            Some(&token),
        )
        .await;
    assert!(status.is_client_error(), "unknown color: {status} {body}");
}

#[sqlx::test]
async fn visual_studio_moves_to_vs_code(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, _) = app.register("vsuser").await;

    // An old client still sending the removed slug lands on its replacement.
    let (status, updated) = app
        .patch(
            SETTINGS_ROUTE,
            json!({ "theme": "visual-studio" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "old slug: {updated}");
    assert_eq!(updated["theme"], "vscode");

    // A row holding the removed slug moves the way the migration moves it.
    sqlx::query("UPDATE user_settings SET theme = 'visual-studio'")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE user_settings SET theme = 'vscode' WHERE theme = 'visual-studio'")
        .execute(&pool)
        .await
        .unwrap();
    let (status, settings) = app.get(SETTINGS_ROUTE, Some(&token)).await;
    assert_eq!(status, StatusCode::OK, "get settings: {settings}");
    assert_eq!(settings["theme"], "vscode");
}

/// A verified user with a live email change to `new@test.local`.
async fn with_pending_change(app: &TestApp, username: &str) -> String {
    let (token, user_id) = app.register(username).await;
    app.verify_email(&user_id).await;
    let (status, body) = request_change(app, &token, "new@test.local").await;
    assert_eq!(status, StatusCode::OK, "change: {body}");
    token
}

/// The pending address and its expiry on a user in a response.
fn pending(user: &serde_json::Value) -> (&serde_json::Value, bool) {
    (
        &user["pending_email"],
        user["pending_email_expires_at"].is_string(),
    )
}

#[sqlx::test]
async fn login_returns_the_pending_email_change(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let _ = with_pending_change(&app, "pendlogin").await;

    let (status, session) = app
        .post(
            LOGIN_ROUTE,
            json!({ "identifier": "pendlogin", "password": "TestPass123!" }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "login: {session}");
    assert_eq!(
        pending(&session["user"]),
        (&json!("new@test.local"), true),
        "the session's user carries the pending change: {session}"
    );
}

#[sqlx::test]
async fn marking_a_hint_keeps_the_pending_email_change(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let token = with_pending_change(&app, "penhint").await;

    let (status, user) = app
        .patch(
            MARK_HINT_SHOWN_ROUTE,
            json!({ "hint": "add_swipes" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "hint: {user}");
    assert_eq!(
        pending(&user),
        (&json!("new@test.local"), true),
        "the clients store this user as the session's: {user}"
    );
}

#[sqlx::test]
async fn changing_the_username_keeps_the_pending_email_change(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let token = with_pending_change(&app, "penname").await;

    let (status, user) = app
        .patch(
            CHANGE_USERNAME_ROUTE,
            json!({ "new_username": "penrenamed", "password": "TestPass123!" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "rename: {user}");
    assert_eq!(pending(&user), (&json!("new@test.local"), true), "{user}");
}
