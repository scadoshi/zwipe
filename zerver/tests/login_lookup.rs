//! Login's account lookup by id, username or email reads the users table
//! through its indexes, so a login's cost doesn't grow with the user count.
//!
//! The repository runs on a one-connection pool, so the seq-scan counter read
//! after each lookup is that connection's own, flushed on demand.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![expect(
    clippy::unwrap_used,
    reason = "a test: helpers outside #[test] fns unwrap, and a panic is the failure report"
)]

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;
use zwipe::domain::auth::models::password::Password;
use zwipe::domain::auth::ports::AuthRepository;
use zwipe::domain::auth::requests::authenticate_user::AuthenticateUser;
use zwipe::outbound::sqlx::postgres::Postgres;

/// Enough users that an index lookup is the planner's choice for one row.
const SEEDED_USERS: i32 = 5_000;

async fn seed_users(pool: &PgPool) {
    let hash = Password::new("TestPass123!").unwrap().hash().unwrap();
    sqlx::query(
        "INSERT INTO users (username, email, password_hash) \
         SELECT 'seed' || n, 'seed' || n || '@test.local', $2 \
         FROM generate_series(1, $1) AS n",
    )
    .bind(SEEDED_USERS)
    .bind(hash.to_string())
    .execute(pool)
    .await
    .unwrap();
    sqlx::query("ANALYZE users").execute(pool).await.unwrap();
}

/// The users table's sequential scans so far, counting this connection's own.
async fn users_seq_scans(single: &PgPool) -> i64 {
    sqlx::query("SELECT pg_stat_force_next_flush()")
        .execute(single)
        .await
        .unwrap();
    sqlx::query_scalar("SELECT seq_scan FROM pg_stat_user_tables WHERE relname = 'users'")
        .fetch_one(single)
        .await
        .unwrap()
}

#[sqlx::test]
async fn login_finds_an_account_by_id_username_or_email_without_scanning_users(pool: PgPool) {
    seed_users(&pool).await;
    let id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE username = 'seed4242'")
        .fetch_one(&pool)
        .await
        .unwrap();
    let single = PgPoolOptions::new()
        .max_connections(1)
        .connect_with((*pool.connect_options()).clone())
        .await
        .unwrap();
    let repo = Postgres::from_pool(single.clone());

    for identifier in [
        "seed4242".to_owned(),
        "seed4242@test.local".to_owned(),
        id.to_string(),
    ] {
        let before = users_seq_scans(&single).await;
        let request = AuthenticateUser::new(&identifier, "unused").unwrap();
        let found = repo.fetch_user_with_password_hash(&request).await.unwrap();
        assert_eq!(found.id, id, "{identifier}");
        assert_eq!(
            users_seq_scans(&single).await,
            before,
            "looking up {identifier} scanned the users table"
        );
    }
}

#[sqlx::test]
async fn only_the_id_s_own_text_finds_an_account_by_id(pool: PgPool) {
    seed_users(&pool).await;
    let id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE username = 'seed7'")
        .fetch_one(&pool)
        .await
        .unwrap();
    let repo = Postgres::from_pool(pool.clone());

    for other_form in [
        id.hyphenated().to_string().to_uppercase(),
        id.simple().to_string(),
        id.braced().to_string(),
    ] {
        let request = AuthenticateUser::new(&other_form, "unused").unwrap();
        assert!(
            repo.fetch_user_with_password_hash(&request).await.is_err(),
            "{other_form} found an account"
        );
    }
}
