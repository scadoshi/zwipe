//! Upkeep (nightly maintenance) prune behavior against real rows — the local
//! zervice runs only proved the grants on empty tables. Constructs
//! `Postgres { pool }` and calls `UpkeepRepository` methods directly.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![allow(clippy::unwrap_used)]

use zwipe::{
    domain::upkeep::{
        ports::UpkeepRepository,
        services::{DIAGNOSTIC_RETENTION_DAYS, REPLACED_TOKEN_RETENTION_DAYS},
    },
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

    let pruned = repo
        .prune_expired_sessions(REPLACED_TOKEN_RETENTION_DAYS)
        .await
        .unwrap();
    assert_eq!(pruned, 2, "both users' expired tokens swept");

    let live: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM refresh_tokens")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(live, 2, "live tokens untouched");
}

#[sqlx::test]
async fn session_prune_kills_replaced_rows_past_grace(pool: sqlx::PgPool) {
    let repo = Postgres { pool: pool.clone() };

    let user_id: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO users (username, email, password_hash)
         VALUES ('rotator', 'rotator@test.local', 'x') RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    // One rotated row just past the grace window, one rotated yesterday, and
    // the live head of the family; all three still inside their 14 days.
    for (hash, replaced_days_ago) in [
        ("stale", Some(REPLACED_TOKEN_RETENTION_DAYS + 1)),
        ("recent", Some(1)),
        ("head", None),
    ] {
        sqlx::query(
            "INSERT INTO refresh_tokens (user_id, value_hash, expires_at, replaced_at)
             VALUES ($1, $2, NOW() + INTERVAL '14 days', NOW() - make_interval(days => $3))",
        )
        .bind(user_id)
        .bind(hash)
        .bind(replaced_days_ago)
        .execute(&pool)
        .await
        .unwrap();
    }

    let pruned = repo
        .prune_expired_sessions(REPLACED_TOKEN_RETENTION_DAYS)
        .await
        .unwrap();
    assert_eq!(pruned, 1, "only the rotated row past the grace window");

    let kept: Vec<String> =
        sqlx::query_scalar("SELECT value_hash FROM refresh_tokens ORDER BY value_hash")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(
        kept,
        ["head", "recent"],
        "recent evidence and the live head stay"
    );
}
