//! Reserved usernames and case: a new name on the reserved list, or one an existing account holds
//! in any case, answers exactly as a taken one on register and on a change; accounts that already
//! hold such names keep them; and the server's admin rename sets a reserved name, records the
//! audit row and mails the account its notice.
//!
//! Requires `DATABASE_URL` (dev's value works): `set -a; source zerver/.env`.

mod common;

use std::sync::Arc;

use axum::http::StatusCode;
use common::{FakeEmailSender, TEST_JWT_SECRET, TestApp};
use serde_json::{Value, json};
use sqlx::PgPool;
use zwipe::domain::auth::models::access_token::JwtSecret;
use zwipe::domain::auth::models::account_ref::AccountRef;
use zwipe::domain::auth::requests::change_username::ChangeUsernameError;
use zwipe::domain::email::models::{SendEmail, SendEmailError};
use zwipe::domain::email::ports::{EmailSender, SendEmailFuture};
use zwipe::domain::{auth, metrics};
use zwipe::inbound::admin::{self, NoticeOutcome, RenameNotice};
use zwipe::outbound::sqlx::postgres::Postgres;
use zwipe::wiring::{Auth, Metrics};
use zwipe_core::domain::Email;
use zwipe_core::domain::user::username::Username;
use zwipe_core::http::paths::{CHANGE_USERNAME_ROUTE, LOGIN_ROUTE, REGISTER_ROUTE};

/// Registers from an address of its own, so the per-address limit never answers instead.
async fn register(pool: &PgPool, username: &str, email: &str) -> (StatusCode, Value) {
    TestApp::new(pool.clone())
        .post(
            REGISTER_ROUTE,
            json!({ "username": username, "email": email, "password": "TestPass123!" }),
            None,
        )
        .await
}

/// Changes the token's username, from an address of its own.
async fn change(pool: &PgPool, token: &str, to: &str) -> (StatusCode, Value) {
    TestApp::new(pool.clone())
        .patch(
            CHANGE_USERNAME_ROUTE,
            json!({ "new_username": to, "password": "TestPass123!" }),
            Some(token),
        )
        .await
}

/// Logs in as `identifier`, from an address of its own.
async fn login(pool: &PgPool, identifier: &str) -> (StatusCode, Value) {
    TestApp::new(pool.clone())
        .post(
            LOGIN_ROUTE,
            json!({ "identifier": identifier, "password": "TestPass123!" }),
            None,
        )
        .await
}

#[expect(
    clippy::unwrap_used,
    reason = "a test helper: a fixture name that fails the rules fails the test"
)]
fn name(raw: &str) -> Username {
    Username::new(raw).unwrap()
}

/// Parses an account the way `zerver-admin` reads its argument.
#[expect(
    clippy::unwrap_used,
    reason = "a test helper: a fixture account that fails to parse fails the test"
)]
fn account(raw: &str) -> AccountRef {
    raw.parse().unwrap()
}

#[expect(
    clippy::unwrap_used,
    reason = "a test helper: a fixture address that fails to parse fails the test"
)]
fn email(raw: &str) -> Email {
    Email::new(raw).unwrap()
}

/// An email sender whose every send fails, as the provider being down does.
#[derive(Clone, Debug)]
struct FailingEmailSender;

impl EmailSender for FailingEmailSender {
    fn send_email(&self, _email: SendEmail) -> SendEmailFuture<'_> {
        Box::pin(async { Err(SendEmailError::ApiError(503, "unavailable".to_owned())) })
    }
}

/// The auth and metrics services `zerver-admin` builds, mailing through `sender`.
#[expect(
    clippy::unwrap_used,
    reason = "a test helper: the fixed test secret is valid"
)]
fn services_sending(pool: &PgPool, sender: Arc<dyn EmailSender>) -> (Auth, Metrics) {
    let db = Postgres::from_pool(pool.clone());
    let auth = auth::services::Service::new(
        db.clone(),
        db.clone(),
        sender,
        JwtSecret::new(TEST_JWT_SECRET).unwrap(),
        chrono::Duration::hours(24),
        "http://localhost".to_owned(),
        "support@zwipe.net".to_owned(),
    );
    (auth, metrics::services::Service::new(db))
}

/// The auth and metrics services `zerver-admin` builds, mailing into a fake sender.
fn services(pool: &PgPool) -> (Auth, Metrics, FakeEmailSender) {
    let emails = FakeEmailSender::default();
    let (auth, metrics) = services_sending(pool, Arc::new(emails.clone()));
    (auth, metrics, emails)
}

#[sqlx::test]
async fn a_reserved_name_registers_exactly_as_a_taken_one(pool: PgPool) {
    let _ = TestApp::new(pool.clone()).register("alice").await;
    let taken = register(&pool, "alice", "other@test.local").await;
    assert_eq!(taken.0, StatusCode::CONFLICT, "{}", taken.1);

    for reserved in [
        "zwipe",
        "Zw1pe",
        "realzwipe",
        "scotty_fermo",
        "admin7",
        "changelog",
    ] {
        let answer = register(&pool, reserved, &format!("{reserved}@test.local")).await;
        assert_eq!(answer, taken, "{reserved} answers as a taken name");
    }
    let users: i64 = sqlx::query_scalar("SELECT count(*) FROM users")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(users, 1, "no reserved name made an account");
}

#[sqlx::test]
async fn a_reserved_name_changes_exactly_as_a_taken_one(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    let _ = app.register("alice").await;
    let (token, _) = app.register("bobby").await;
    let taken = change(&pool, &token, "alice").await;
    assert_eq!(taken.0, StatusCode::CONFLICT, "{}", taken.1);

    for reserved in ["zwipe", "the_scotty", "Support", "decks"] {
        assert_eq!(
            change(&pool, &token, reserved).await,
            taken,
            "{reserved} answers as a taken name"
        );
    }
    let (status, body) = change(&pool, &token, "bobby_two").await;
    assert_eq!(status, StatusCode::OK, "an open name still changes: {body}");
}

#[sqlx::test]
async fn a_new_name_is_taken_by_an_existing_one_in_any_case(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    let _ = app.register("alice").await;
    let taken = register(&pool, "alice", "other@test.local").await;
    assert_eq!(
        register(&pool, "ALICE", "upper@test.local").await,
        taken,
        "a register in another case answers as taken"
    );

    let (token, _) = app.register("bobby").await;
    let taken_change = change(&pool, &token, "alice").await;
    assert_eq!(taken_change.0, StatusCode::CONFLICT);
    assert_eq!(change(&pool, &token, "Alice").await, taken_change);
    let (status, body) = change(&pool, &token, "Bobby").await;
    assert_eq!(
        status,
        StatusCode::OK,
        "its own name in another case: {body}"
    );
}

#[sqlx::test]
async fn an_account_keeps_a_reserved_name_it_already_holds(pool: PgPool) {
    let (token, _) = TestApp::new(pool.clone()).register("placeholder").await;
    let (auth, metrics, _) = services(&pool);
    admin::rename_account(
        &auth,
        &metrics,
        &account("placeholder"),
        &name("Scotty"),
        RenameNotice::Skip,
    )
    .await
    .unwrap();

    let (status, session) = login(&pool, "Scotty").await;
    assert_eq!(status, StatusCode::OK, "{session}");
    assert_eq!(session["user"]["username"], "Scotty");

    let (status, body) = change(&pool, &token, "scotty").await;
    assert_eq!(
        status,
        StatusCode::OK,
        "the holder may rewrite its own name: {body}"
    );
    assert_eq!(body["username"], "scotty");
}

#[sqlx::test]
async fn names_that_already_differ_only_in_case_both_stay(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    let _ = app.register("pair").await;
    let (token, _) = app.register("second").await;
    sqlx::query("UPDATE users SET username = 'Pair' WHERE username = 'second'")
        .execute(&pool)
        .await
        .unwrap();

    for identifier in ["pair", "Pair"] {
        let (status, session) = login(&pool, identifier).await;
        assert_eq!(status, StatusCode::OK, "{identifier}: {session}");
        assert_eq!(session["user"]["username"], identifier);
    }

    let (status, _) = change(&pool, &token, "PAIR").await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "a third case of a held name is taken"
    );
    let (status, body) = change(&pool, &token, "pair_two").await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

#[sqlx::test]
async fn the_admin_rename_sets_a_reserved_name_audits_and_mails_the_notice(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    let _ = app.register("alice").await;
    // "Test" is reserved now, so the account that holds it from before is made by hand.
    let _ = app.register("tester").await;
    sqlx::query("UPDATE users SET username = 'Test' WHERE username = 'tester'")
        .execute(&pool)
        .await
        .unwrap();
    let (auth, metrics, emails) = services(&pool);

    let renamed = admin::rename_account(
        &auth,
        &metrics,
        &account("Test"),
        &name("naxol56196"),
        RenameNotice::Send {
            reply_to: email("scotty@zwipe.net"),
        },
    )
    .await
    .unwrap();
    assert_eq!(&*renamed.old_username, "Test");
    assert_eq!(&*renamed.user.username, "naxol56196");
    assert!(matches!(renamed.notice, NoticeOutcome::Sent));

    let audits: Vec<String> = sqlx::query_scalar(
        "SELECT action FROM user_audit_log WHERE user_id = $1 AND action <> 'login'",
    )
    .bind(renamed.user.id)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(audits, ["username_changed"]);

    {
        let sent = emails.sent.lock().unwrap();
        let notice = sent.last().unwrap();
        assert_eq!(notice.to, "tester@test.local");
        assert_eq!(notice.subject, "Your Zwipe username has changed");
        assert_eq!(notice.reply_to.as_deref(), Some("scotty@zwipe.net"));
        let text = notice.text_body.as_deref().unwrap();
        assert!(
            text.contains(
                "We've reserved the username \"Test\" for Zwipe's own use, so your account's username is now naxol56196."
            ),
            "{text}"
        );
        assert!(text.trim_end().ends_with("Scotty"), "{text}");
        assert!(notice.html_body.contains("naxol56196"));
    }

    let (status, _) = login(&pool, "naxol56196").await;
    assert_eq!(
        status,
        StatusCode::OK,
        "the account signs in under its new name"
    );
}

#[sqlx::test]
async fn the_admin_rename_finds_an_account_by_id_and_can_skip_the_notice(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    let (_, user_id) = app.register("carol").await;
    let _ = app.register("alice").await;
    let (auth, metrics, emails) = services(&pool);
    let before = emails.count();

    let renamed = admin::rename_account(
        &auth,
        &metrics,
        &account(&user_id),
        &name("zwipe"),
        RenameNotice::Skip,
    )
    .await
    .unwrap();
    assert_eq!(&*renamed.old_username, "carol");
    assert!(matches!(renamed.notice, NoticeOutcome::Skipped));
    assert_eq!(emails.count(), before, "no notice went out");

    let taken = admin::rename_account(
        &auth,
        &metrics,
        &account("zwipe"),
        &name("Alice"),
        RenameNotice::Skip,
    )
    .await;
    assert!(
        matches!(taken, Err(ChangeUsernameError::Duplicate)),
        "a name another account holds in any case: {taken:?}"
    );
    let missing = admin::rename_account(
        &auth,
        &metrics,
        &account("nobody_here"),
        &name("scott"),
        RenameNotice::Skip,
    )
    .await;
    assert!(
        matches!(missing, Err(ChangeUsernameError::AuthenticateUserError(_))),
        "no such account: {missing:?}"
    );
}

#[sqlx::test]
async fn the_admin_rename_finds_an_account_by_email(pool: PgPool) {
    let _ = TestApp::new(pool.clone()).register("dana").await;
    let (auth, metrics, emails) = services(&pool);

    let renamed = admin::rename_account(
        &auth,
        &metrics,
        &account("dana@test.local"),
        &name("dana_two"),
        RenameNotice::Send {
            reply_to: email("support@zwipe.net"),
        },
    )
    .await
    .unwrap();
    assert_eq!(&*renamed.old_username, "dana");
    assert_eq!(&*renamed.user.username, "dana_two");
    assert!(matches!(renamed.notice, NoticeOutcome::Sent));

    let sent = emails.sent.lock().unwrap();
    let notice = sent.last().unwrap();
    assert_eq!(notice.to, "dana@test.local");
    assert_eq!(notice.reply_to.as_deref(), Some("support@zwipe.net"));
}

#[sqlx::test]
async fn an_email_address_no_account_holds_finds_nothing(pool: PgPool) {
    let _ = TestApp::new(pool.clone()).register("erin").await;
    let (auth, metrics, _) = services(&pool);
    let missing = admin::rename_account(
        &auth,
        &metrics,
        &account("someone@test.local"),
        &name("erin_two"),
        RenameNotice::Skip,
    )
    .await;
    assert!(
        matches!(missing, Err(ChangeUsernameError::AuthenticateUserError(_))),
        "{missing:?}"
    );
}

#[sqlx::test]
async fn the_rename_stands_when_the_notice_fails(pool: PgPool) {
    let _ = TestApp::new(pool.clone()).register("frank").await;
    let (auth, metrics) = services_sending(&pool, Arc::new(FailingEmailSender));

    let renamed = admin::rename_account(
        &auth,
        &metrics,
        &account("frank"),
        &name("frank_two"),
        RenameNotice::Send {
            reply_to: email("support@zwipe.net"),
        },
    )
    .await
    .unwrap();
    assert!(
        matches!(renamed.notice, NoticeOutcome::Failed(_)),
        "{:?}",
        renamed.notice
    );

    let held: String = sqlx::query_scalar("SELECT username FROM users WHERE id = $1")
        .bind(renamed.user.id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(held, "frank_two", "the rename went through");
    let (status, _) = login(&pool, "frank_two").await;
    assert_eq!(status, StatusCode::OK);
}
