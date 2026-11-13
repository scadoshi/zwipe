//! The `zerver-admin` commands beyond rename, run through the services the binary builds with a
//! fake email sender: show reads an account at a glance, sign-out-all revokes its refresh tokens,
//! resend-verification mails a fresh link, and delete removes it, each writing its audit row
//! where the account survives.
//!
//! Requires `DATABASE_URL` (dev's value works): `set -a; source zerver/.env`.

mod common;

use std::sync::Arc;

use common::{FakeEmailSender, TEST_JWT_SECRET, TestApp};
use sqlx::PgPool;
use uuid::Uuid;
use zwipe::domain::auth::models::access_token::JwtSecret;
use zwipe::domain::auth::models::account_ref::AccountRef;
use zwipe::domain::auth::ports::AuthService;
use zwipe::domain::auth::requests::authenticate_user::AuthenticateUserError;
use zwipe::domain::{auth, metrics, user};
use zwipe::inbound::admin::{self, ResendVerificationError};
use zwipe::outbound::sqlx::postgres::Postgres;
use zwipe::wiring::{Auth, Metrics, Users};
use zwipe_core::domain::user::User;

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
                Err(AuthenticateUserError::UserNotFound)
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

    let summary = admin::show_account(&users, &hank).await.unwrap();
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
    let _ = admin::show_account(&users, &hank).await.unwrap();
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
    let (auth, ..) = services(&pool);
    let kate = find(&auth, "kate").await;

    admin::delete_account(&auth, &kate).await.unwrap();
    let left: Vec<String> = sqlx::query_scalar("SELECT username FROM users ORDER BY username")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(left, ["lars"]);
    let account: AccountRef = "kate".parse().unwrap();
    assert!(matches!(
        auth.fetch_account(&account).await,
        Err(AuthenticateUserError::UserNotFound)
    ));
}
