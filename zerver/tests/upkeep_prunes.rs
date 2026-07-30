//! Upkeep (nightly maintenance) prune behavior against real rows — the local
//! zervice runs only proved the grants on empty tables. Constructs
//! `Postgres { pool }` and calls `UpkeepRepository` methods directly.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![allow(clippy::unwrap_used)]

use zwipe::{
    domain::upkeep::{ports::UpkeepRepository, services::DIAGNOSTIC_RETENTION_DAYS},
    outbound::sqlx::postgres::Postgres,
};

/// Inserts a client_errors row with `received_at` backdated by `days`.
async fn seed_client_error(pool: &sqlx::PgPool, days: i32) {
    sqlx::query(
        "INSERT INTO client_errors
             (received_at, client_version, platform, screen, component, action, kind, message, count)
         VALUES (NOW() - make_interval(days => $1), '1.8.0', 'ios', 'deck_edit', '', 'save', 'api_internal', 'boom', 1)",
    )
    .bind(days)
    .execute(pool)
    .await
    .unwrap();
}

/// Inserts a crash_reports row with `received_at` backdated by `days`.
async fn seed_crash(pool: &sqlx::PgPool, days: i32) {
    sqlx::query(
        "INSERT INTO crash_reports
             (crash_id, received_at, occurred_at, client_version, platform, message)
         VALUES (gen_random_uuid(), NOW() - make_interval(days => $1),
                 NOW() - make_interval(days => $1), '1.8.0', 'android', 'panicked')",
    )
    .bind(days)
    .execute(pool)
    .await
    .unwrap();
}

#[sqlx::test]
async fn diagnostic_prunes_kill_old_and_spare_fresh(pool: sqlx::PgPool) {
    let repo = Postgres { pool: pool.clone() };

    // One row just past the window, one comfortably inside it, per table.
    seed_client_error(&pool, DIAGNOSTIC_RETENTION_DAYS + 1).await;
    seed_client_error(&pool, 1).await;
    seed_crash(&pool, DIAGNOSTIC_RETENTION_DAYS + 1).await;
    seed_crash(&pool, 1).await;

    let pruned = repo
        .prune_client_errors(DIAGNOSTIC_RETENTION_DAYS)
        .await
        .unwrap();
    assert_eq!(pruned, 1, "exactly the expired client_errors row");
    let pruned = repo
        .prune_crash_reports(DIAGNOSTIC_RETENTION_DAYS)
        .await
        .unwrap();
    assert_eq!(pruned, 1, "exactly the expired crash_reports row");

    let errors: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM client_errors")
        .fetch_one(&pool)
        .await
        .unwrap();
    let crashes: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM crash_reports")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!((errors, crashes), (1, 1), "fresh rows survive");
}

#[sqlx::test]
async fn session_prune_kills_expired_across_users(pool: sqlx::PgPool) {
    let repo = Postgres { pool: pool.clone() };

    // Two dormant users, expired + live tokens each — the global sweep case
    // the insert-time drive-by never reaches.
    for who in ["ghost_one", "ghost_two"] {
        let user_id: uuid::Uuid = sqlx::query_scalar(
            "INSERT INTO users (username, email, password_hash)
             VALUES ($1, $1 || '@test.local', 'x') RETURNING id",
        )
        .bind(who)
        .fetch_one(&pool)
        .await
        .unwrap();
        for (hash, offset_days) in [("expired", -1), ("live", 14)] {
            sqlx::query(
                "INSERT INTO refresh_tokens (user_id, value_hash, expires_at)
                 VALUES ($1, $2, NOW() + make_interval(days => $3))",
            )
            .bind(user_id)
            .bind(format!("{who}-{hash}"))
            .bind(offset_days)
            .execute(&pool)
            .await
            .unwrap();
        }
    }

    let pruned = repo.prune_expired_sessions().await.unwrap();
    assert_eq!(pruned, 2, "both users' expired tokens swept");

    let live: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM refresh_tokens")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(live, 2, "live tokens untouched");
}
