//! The `zerver-admin` commands beyond rename, run through the services the binary builds with a
//! fake email sender: show reads an account at a glance, sign-out-all revokes its refresh tokens,
//! resend-verification mails a fresh link, and delete removes it, each writing its audit row
//! where the account survives.
//!
//! Requires `DATABASE_URL` (dev's value works): `set -a; source zerver/.env`.

mod common;

use std::sync::Arc;

use axum::http::StatusCode;
use common::{FakeEmailSender, TEST_JWT_SECRET, TestApp};
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;
use zwipe::domain::auth::models::access_token::JwtSecret;
use zwipe::domain::auth::models::account_ref::{AccountRef, FetchAccountError};
use zwipe::domain::auth::ports::AuthService;
use zwipe::domain::auth::requests::admin_change_email::AdminChangeEmailError;
use zwipe::domain::{auth, metrics, user};
use zwipe::inbound::admin::{self, NoticeOutcome, ResendVerificationError};
use zwipe::outbound::sqlx::postgres::Postgres;
use zwipe::wiring::{Auth, Metrics, Users};
use zwipe_core::domain::Email;
use zwipe_core::domain::user::User;
use zwipe_core::http::paths::{LOGIN_ROUTE, REFRESH_SESSION_ROUTE};

/// The services `zerver-admin` builds, mailing into a fake sender.
#[expect(
    clippy::unwrap_used,
    reason = "a test helper: the fixed test secret is valid"
)]
fn services(pool: &PgPool) -> (Auth, Users, Metrics, FakeEmailSender) {
    let db = Postgres::from_pool(pool.clone());
    let emails = FakeEmailSender::default();
    let auth = auth::services::Service::new(
        db.clone(),
        db.clone(),
        Arc::new(emails.clone()),
        JwtSecret::new(TEST_JWT_SECRET).unwrap(),
        chrono::Duration::hours(24),
        "http://localhost".to_owned(),
        "support@zwipe.net".to_owned(),
    );
    (
        auth,
        user::services::Service::new(db.clone()),
        metrics::services::Service::new(db),
        emails,
    )
}

/// The account `raw` names, as `zerver-admin` finds it.
#[expect(
    clippy::unwrap_used,
    reason = "a test helper: a fixture account that is missing fails the test"
)]
async fn find(auth: &Auth, raw: &str) -> User {
    let account: AccountRef = raw.parse().unwrap();
    auth.fetch_account(&account).await.unwrap()
}

/// `user_id`'s audit actions other than the login and refresh rows sign-in writes, oldest first.
#[expect(
    clippy::unwrap_used,
    reason = "a test helper: a failed query fails the test"
)]
async fn admin_audits(pool: &PgPool, user_id: Uuid) -> Vec<String> {
    sqlx::query_scalar(
        "SELECT action FROM user_audit_log WHERE user_id = $1 AND action NOT IN ('login', 'refresh') ORDER BY id",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .unwrap()
}

#[sqlx::test]
async fn an_account_is_found_by_username_id_or_email_and_not_otherwise(pool: PgPool) {
    let (_, user_id) = TestApp::new(pool.clone()).register("gina").await;
    let (auth, ..) = services(&pool);
    for raw in ["gina", user_id.as_str(), "gina@test.local"] {
        assert_eq!(&*find(&auth, raw).await.username, "gina", "{raw}");
    }
    for raw in ["Gina", "nobody_here", "nobody@test.local"] {
        let account: AccountRef = raw.parse().unwrap();
        assert!(
            matches!(
                auth.fetch_account(&account).await,
                Err(FetchAccountError::NotFound(_))
            ),
            "{raw}"
        );
    }
}

#[sqlx::test]
async fn show_reads_the_account_its_decks_and_its_latest_audit_events(pool: PgPool) {
    let (_, user_id) = TestApp::new(pool.clone()).register("hank").await;
    let (auth, users, metrics, _) = services(&pool);
    let hank = find(&auth, "hank").await;
    for name in ["one", "two"] {
        sqlx::query("INSERT INTO decks (name, user_id) VALUES ($1, $2)")
            .bind(name)
            .bind(hank.id)
            .execute(&pool)
            .await
            .unwrap();
    }
    admin::sign_out_all(&auth, &metrics, &hank).await.unwrap();

    let summary = admin::show_account(&users, &metrics, &hank).await.unwrap();
    assert_eq!(summary.user.id.to_string(), user_id);
    assert_eq!(&*summary.user.username, "hank");
    assert_eq!(summary.user.email.as_ref(), "hank@test.local");
    assert_eq!(summary.deck_count, 2);
    assert_eq!(
        summary
            .recent_audit
            .first()
            .map(|entry| entry.action.as_str()),
        Some("signed_out_everywhere"),
        "newest first: {:?}",
        summary.recent_audit
    );
    let audits_before = admin_audits(&pool, hank.id).await;
    let _ = admin::show_account(&users, &metrics, &hank).await.unwrap();
    assert_eq!(
        admin_audits(&pool, hank.id).await,
        audits_before,
        "show writes nothing"
    );
}

#[sqlx::test]
async fn sign_out_all_revokes_every_refresh_token_and_audits(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    let _ = app.register("ivy").await;
    let (_, other_id) = app.register("other").await;
    let (auth, _, metrics, _) = services(&pool);
    let ivy = find(&auth, "ivy").await;
    let tokens = |id: Uuid| {
        let pool = pool.clone();
        async move {
            sqlx::query_scalar::<_, i64>("SELECT count(*) FROM refresh_tokens WHERE user_id = $1")
                .bind(id)
                .fetch_one(&pool)
                .await
                .unwrap()
        }
    };
    assert!(tokens(ivy.id).await > 0, "registering signed in");

    admin::sign_out_all(&auth, &metrics, &ivy).await.unwrap();
    assert_eq!(tokens(ivy.id).await, 0);
    let other: Uuid = other_id.parse().unwrap();
    assert!(
        tokens(other).await > 0,
        "another account keeps its sessions"
    );
    assert_eq!(admin_audits(&pool, ivy.id).await, ["signed_out_everywhere"]);
}

#[sqlx::test]
async fn resend_verification_mails_a_fresh_link_once_and_audits(pool: PgPool) {
    let _ = TestApp::new(pool.clone()).register("jack").await;
    let (auth, _, metrics, emails) = services(&pool);
    let jack = find(&auth, "jack").await;

    admin::resend_verification(&auth, &metrics, &jack)
        .await
        .unwrap();
    assert_eq!(emails.count(), 1);
    assert_eq!(emails.last_to().as_deref(), Some("jack@test.local"));
    assert!(
        emails.last_token("verify").is_some(),
        "the email carries a verify link"
    );
    assert_eq!(
        admin_audits(&pool, jack.id).await,
        ["verification_email_sent"]
    );

    sqlx::query("UPDATE users SET email_verified_at = NOW() WHERE id = $1")
        .bind(jack.id)
        .execute(&pool)
        .await
        .unwrap();
    let verified = find(&auth, "jack").await;
    let again = admin::resend_verification(&auth, &metrics, &verified).await;
    assert!(
        matches!(again, Err(ResendVerificationError::AlreadyVerified)),
        "{again:?}"
    );
    assert_eq!(emails.count(), 1, "a verified address gets nothing");
}

#[sqlx::test]
async fn delete_removes_the_account_and_only_that_account(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    let _ = app.register("kate").await;
    let _ = app.register("lars").await;
    let (auth, _, metrics, _) = services(&pool);
    let kate = find(&auth, "kate").await;

    admin::delete_account(&auth, &metrics, &kate).await.unwrap();
    let left: Vec<String> = sqlx::query_scalar("SELECT username FROM users ORDER BY username")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(left, ["lars"]);
    let account: AccountRef = "kate".parse().unwrap();
    assert!(matches!(
        auth.fetch_account(&account).await,
        Err(FetchAccountError::NotFound(_))
    ));
}

/// Signs in as `identifier` with `password`, from an address of its own.
async fn login_with(pool: &PgPool, identifier: &str, password: &str) -> (StatusCode, Value) {
    TestApp::new(pool.clone())
        .post(
            LOGIN_ROUTE,
            json!({ "identifier": identifier, "password": password }),
            None,
        )
        .await
}

/// Refreshes `session`, from an address of its own.
#[expect(
    clippy::indexing_slicing,
    reason = "a test helper: JSON indexing answers null for a missing key"
)]
async fn refresh(pool: &PgPool, session: &Value) -> (StatusCode, Value) {
    TestApp::new(pool.clone())
        .post(
            REFRESH_SESSION_ROUTE,
            json!({
                "user_id": session["user"]["id"],
                "refresh_token": session["refresh_token"]["value"],
            }),
            None,
        )
        .await
}

/// `account_id`'s `admin_action_log` actions, oldest first.
#[expect(
    clippy::unwrap_used,
    reason = "a test helper: a failed query fails the test"
)]
async fn admin_actions(pool: &PgPool, account_id: Uuid) -> Vec<String> {
    sqlx::query_scalar("SELECT action FROM admin_action_log WHERE account_id = $1 ORDER BY id")
        .bind(account_id)
        .fetch_all(pool)
        .await
        .unwrap()
}

#[sqlx::test]
async fn a_lock_refuses_sign_in_and_refresh_as_a_wrong_password_does_until_unlocked(pool: PgPool) {
    let _ = TestApp::new(pool.clone()).register("mona").await;
    let (auth, users, metrics, _) = services(&pool);
    let mona = find(&auth, "mona").await;
    let (status, session) = login_with(&pool, "mona", "TestPass123!").await;
    assert_eq!(status, StatusCode::OK, "{session}");
    let wrong_password = login_with(&pool, "mona", "WrongPass123!").await;
    assert_eq!(wrong_password.0, StatusCode::UNAUTHORIZED);

    admin::lock_account(&auth, &metrics, &mona, Some("chargeback"))
        .await
        .unwrap();
    assert_eq!(
        login_with(&pool, "mona", "TestPass123!").await,
        wrong_password,
        "the right password answers as a wrong one, as the lockout does"
    );
    let (status, _) = refresh(&pool, &session).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "sessions stop refreshing");
    let summary = admin::show_account(&users, &metrics, &mona).await.unwrap();
    let lock = summary.lock.unwrap();
    assert_eq!(lock.reason.as_deref(), Some("chargeback"));

    admin::unlock_account(&auth, &metrics, &mona).await.unwrap();
    let (status, body) = login_with(&pool, "mona", "TestPass123!").await;
    assert_eq!(status, StatusCode::OK, "signs in again: {body}");
    assert!(
        admin::show_account(&users, &metrics, &mona)
            .await
            .unwrap()
            .lock
            .is_none()
    );
    assert_eq!(
        admin_audits(&pool, mona.id).await,
        ["account_locked", "account_unlocked"]
    );
    assert_eq!(
        admin_actions(&pool, mona.id).await,
        ["lock", "show", "unlock", "show"]
    );
}

#[sqlx::test]
async fn the_admin_action_log_outlives_a_deleted_account(pool: PgPool) {
    let _ = TestApp::new(pool.clone()).register("nina").await;
    let (auth, users, metrics, _) = services(&pool);
    let nina = find(&auth, "nina").await;
    let _ = admin::show_account(&users, &metrics, &nina).await.unwrap();
    admin::sign_out_all(&auth, &metrics, &nina).await.unwrap();
    admin::delete_account(&auth, &metrics, &nina).await.unwrap();
    assert_eq!(
        admin_actions(&pool, nina.id).await,
        ["show", "sign_out_all", "delete"]
    );
    let names: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM information_schema.columns WHERE table_name = 'admin_action_log' AND column_name IN ('username', 'email')",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(names, 0, "the log names no username or email");
}

#[sqlx::test]
async fn change_email_switches_at_once_verifies_the_new_address_and_tells_the_old(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    let _ = app.register("olga").await;
    let _ = app.register("pete").await;
    let (auth, _, metrics, emails) = services(&pool);
    let olga = find(&auth, "olga").await;
    let new = Email::new("olga.new@test.local").unwrap();
    let reply_to = Email::new("support@zwipe.net").unwrap();

    let switched = admin::change_email(&auth, &metrics, &olga, &new, &reply_to)
        .await
        .unwrap();
    assert_eq!(switched.old_email.as_ref(), "olga@test.local");
    assert_eq!(switched.user.email.as_ref(), "olga.new@test.local");
    assert!(
        switched.user.email_verified_at.is_none(),
        "the new address is unverified"
    );
    assert!(matches!(switched.verification, NoticeOutcome::Sent));
    assert!(matches!(switched.notice, NoticeOutcome::Sent));

    {
        let sent = emails.sent.lock().unwrap();
        let [verify, notice] = sent.as_slice() else {
            panic!("two emails: {sent:?}");
        };
        assert_eq!(verify.to, "olga.new@test.local");
        assert!(verify.html_body.contains("/verify/"), "a verify link");
        assert_eq!(notice.to, "olga@test.local");
        assert_eq!(notice.subject, "Your Zwipe email was changed");
        assert_eq!(notice.reply_to.as_deref(), Some("support@zwipe.net"));
        assert!(notice.html_body.contains("olga.new@test.local"));
    }
    assert_eq!(admin_audits(&pool, olga.id).await, ["email_changed"]);
    assert_eq!(admin_actions(&pool, olga.id).await, ["change_email"]);
    let (status, _) = login_with(&pool, "olga.new@test.local", "TestPass123!").await;
    assert_eq!(status, StatusCode::OK, "signs in at the new address");

    let moved = find(&auth, "olga").await;
    let taken = Email::new("pete@test.local").unwrap();
    assert!(matches!(
        admin::change_email(&auth, &metrics, &moved, &taken, &reply_to).await,
        Err(AdminChangeEmailError::Taken)
    ));
    assert!(matches!(
        admin::change_email(&auth, &metrics, &moved, &new, &reply_to).await,
        Err(AdminChangeEmailError::Unchanged)
    ));
    assert_eq!(emails.count(), 2, "a refused change mails nothing");
}
