//! zervice's dry run against a real database:
//! with [`Writes::Discard`] every step's write runs and reports what it would change, and nothing
//! changes. The same writes with [`Writes::Keep`] change the rows.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![expect(clippy::unwrap_used, reason = "a test: a panic is the failure report")]

mod common;

use std::collections::BTreeMap;

use chrono::Utc;
use common::{card, seed_cards};
use uuid::Uuid;
use zwipe::domain::card::models::zervice_metrics::ZerviceMetrics;
use zwipe::domain::card::models::zervice_step::{Step, StepOutcome, StepRecord};
use zwipe::domain::card::ports::CardRepository;
use zwipe::domain::card::requests::get_scryfall_data::GetScryfallData;
use zwipe::domain::upkeep::ports::UpkeepRepository;
use zwipe::outbound::sqlx::postgres::{Postgres, Writes};

/// A `zervice_steps` row as the test reads it: step, status, rows, counts, error, dry run and
/// duration.
type StepRow = (
    String,
    String,
    Option<i64>,
    serde_json::Value,
    Option<String>,
    bool,
    i64,
);

/// Seeds `n` client errors received 100 days ago, past the 90-day retention.
async fn old_client_errors(pool: &sqlx::PgPool, n: i64) {
    sqlx::query(
        "INSERT INTO client_errors (received_at, client_version, platform, screen, action, kind, message) \
         SELECT now() - interval '100 days', '1.0.0', 'web', 'home', 'load', 'network', 'x' \
         FROM generate_series(1, $1)",
    )
    .bind(n)
    .execute(pool)
    .await
    .unwrap();
}

async fn client_errors(pool: &sqlx::PgPool) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM client_errors")
        .fetch_one(pool)
        .await
        .unwrap()
}

#[sqlx::test]
async fn a_dry_run_prune_counts_what_it_would_delete_and_deletes_nothing(pool: sqlx::PgPool) {
    old_client_errors(&pool, 3).await;
    let dry = Postgres::from_pool(pool.clone()).with_writes(Writes::Discard);
    assert_eq!(
        dry.prune_client_errors(90).await.unwrap(),
        3,
        "it counts the three"
    );
    assert_eq!(client_errors(&pool).await, 3, "and keeps them");

    let real = Postgres::from_pool(pool.clone());
    assert_eq!(real.prune_client_errors(90).await.unwrap(), 3);
    assert_eq!(client_errors(&pool).await, 0, "a real run deletes them");
}

#[sqlx::test]
async fn a_dry_run_card_sync_upserts_nothing(pool: sqlx::PgPool) {
    let fixture = card("Llanowar Elves");
    seed_cards(&pool, std::slice::from_ref(&fixture)).await;
    let real = Postgres::from_pool(pool.clone());
    let mut renamed = real
        .fetch_scryfall_data(&GetScryfallData::new(&fixture.id().to_string()).unwrap())
        .await
        .unwrap();
    renamed.name = "Llanowar Elves (renamed)".to_owned();
    let name = |pool: sqlx::PgPool| async move {
        sqlx::query_scalar::<_, String>("SELECT name FROM scryfall_data")
            .fetch_one(&pool)
            .await
            .unwrap()
    };

    let dry = real.clone().with_writes(Writes::Discard);
    let mut metrics = ZerviceMetrics::new();
    let _ = dry
        .batch_delta_upsert(std::slice::from_ref(&renamed), 100, &mut metrics)
        .await
        .unwrap();
    assert_eq!(
        name(pool.clone()).await,
        "Llanowar Elves",
        "the dry run kept the old name"
    );

    let mut metrics = ZerviceMetrics::new();
    let _ = real
        .batch_delta_upsert(std::slice::from_ref(&renamed), 100, &mut metrics)
        .await
        .unwrap();
    assert_eq!(name(pool.clone()).await, "Llanowar Elves (renamed)");
}

#[sqlx::test]
async fn a_dry_run_tag_projection_reports_rows_and_changes_none(pool: sqlx::PgPool) {
    // A tagged profile whose card has no tag rows: the projection would reset it to [].
    seed_cards(&pool, &[card("Sol Ring").oracle_tags(&["ramp"])]).await;
    let tags = |pool: sqlx::PgPool| async move {
        sqlx::query_scalar::<_, serde_json::Value>("SELECT oracle_tags FROM card_profiles")
            .fetch_one(&pool)
            .await
            .unwrap()
    };
    let dry = Postgres::from_pool(pool.clone()).with_writes(Writes::Discard);
    assert_eq!(
        dry.refresh_card_oracle_tags().await.unwrap(),
        1,
        "one profile would change"
    );
    assert_eq!(
        tags(pool.clone()).await,
        serde_json::json!(["ramp"]),
        "and none did"
    );

    let real = Postgres::from_pool(pool.clone());
    assert_eq!(real.refresh_card_oracle_tags().await.unwrap(), 1);
    assert_eq!(tags(pool.clone()).await, serde_json::json!([]));
}

#[sqlx::test]
async fn every_step_records_a_row_a_dry_runs_included(pool: sqlx::PgPool) {
    let run_id = Uuid::new_v4();
    let dry = Postgres::from_pool(pool.clone()).with_writes(Writes::Discard);
    for (step, outcome) in [
        (
            Step::OracleTags,
            StepOutcome::Ok {
                rows: Some(1830),
                counts: BTreeMap::from([("tags", 1830), ("correlations", 41_202)]),
            },
        ),
        (Step::RefreshLatestCards, StepOutcome::Skipped("dry run")),
        (
            Step::PruneClientErrors,
            StepOutcome::Failed("boom".to_owned()),
        ),
    ] {
        dry.record_zervice_step(&StepRecord {
            run_id,
            step,
            started_at: Utc::now(),
            duration_ms: 12,
            outcome,
            dry_run: true,
        })
        .await
        .unwrap();
    }
    let rows: Vec<StepRow> = sqlx::query_as(
        "SELECT step, status, rows, counts, error, dry_run, duration_ms FROM zervice_steps \
             WHERE run_id = $1 ORDER BY step",
    )
    .bind(run_id)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        rows,
        vec![
            (
                "oracle_tags".to_owned(),
                "ok".to_owned(),
                Some(1830),
                serde_json::json!({ "tags": 1830, "correlations": 41_202 }),
                None,
                true,
                12,
            ),
            (
                "prune_client_errors".to_owned(),
                "failed".to_owned(),
                None,
                serde_json::json!({}),
                Some("boom".to_owned()),
                true,
                12,
            ),
            (
                "refresh_latest_cards".to_owned(),
                "skipped".to_owned(),
                None,
                serde_json::json!({ "skipped": "dry run" }),
                None,
                true,
                12,
            ),
        ],
        "a dry run keeps its step rows, flagged, though its writes roll back"
    );
}
