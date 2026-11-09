//! `zcripts/seed/activity.sql`, the local-only seed that gives an account a
//! busy year of Activity for the marketing clip: it fills every Activity
//! panel, keeps the usage tables in step with each other, replaces rather
//! than adds on a second run, and its wrapper refuses a database that isn't
//! local.

#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    reason = "a test: fixtures unwrap, index and count up from small numbers, and a panic is the failure report"
)]

mod common;

use std::process::Command;

use axum::http::StatusCode;
use common::{TestApp, card, seed_cards};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;
use zwipe_core::http::paths::ACTIVITY_ROUTE;

/// The seed, as the wrapper runs it.
const SEED_SQL: &str = include_str!("../../zcripts/seed/activity.sql");

/// The card roles the fixture cards cycle through.
const ROLES: [&str; 6] = ["ramp", "removal", "draw", "tokens", "counters", "lifegain"];

/// A catalog of 60 role-carrying cards and two commanders.
async fn catalog(pool: &PgPool) {
    let cards: Vec<_> = (0..60)
        .map(|i| {
            card(&format!("Seed Card {i}"))
                .categories(&[ROLES[i % ROLES.len()]])
                .edhrec_rank(i32::try_from(i).unwrap() + 100)
        })
        .collect();
    seed_cards(pool, &cards).await;
    let leaders = [
        card("Seed Leader One")
            .type_line("Legendary Creature — Human")
            .edhrec_rank(1)
            .commander_legal(),
        card("Seed Leader Two")
            .type_line("Legendary Creature — Elf")
            .edhrec_rank(2)
            .commander_legal(),
    ];
    seed_cards(pool, &leaders).await;
}

/// Runs the seed for `account` with the default random seed.
async fn seed(pool: &PgPool, account: &str) -> Result<(), sqlx::Error> {
    let mut conn = pool.acquire().await.unwrap();
    sqlx::query("SELECT set_config('zwipe.seed_user', $1, false)")
        .bind(account)
        .execute(&mut *conn)
        .await
        .unwrap();
    sqlx::raw_sql(SEED_SQL)
        .execute(&mut *conn)
        .await
        .map(|_| ())
}

/// Every table's totals for `user`: lifetime, daily, weekly and per card.
async fn totals(pool: &PgPool, user: Uuid) -> Value {
    sqlx::query_scalar(
        "SELECT jsonb_build_object(
            'lifetime', (SELECT swipes_right + swipes_left + swipes_up + swipes_down
                         FROM user_lifetime_counters WHERE user_id = $1),
            'lifetime_searches', (SELECT searches FROM user_lifetime_counters WHERE user_id = $1),
            'daily', (SELECT sum(swipes_right + swipes_left + swipes_up + swipes_down)
                      FROM user_daily_activity WHERE user_id = $1),
            'daily_searches', (SELECT sum(searches) FROM user_daily_activity WHERE user_id = $1),
            'days', (SELECT count(*) FROM user_daily_activity WHERE user_id = $1),
            'weekly', (SELECT sum(swipes_right + swipes_left + swipes_up + swipes_down)
                       FROM user_week_signal WHERE user_id = $1),
            'weekly_outcomes', (SELECT sum(added + skipped + maybed + removed)
                                FROM user_week_signal WHERE user_id = $1),
            'card_outcomes', (SELECT sum(added + skipped + maybed + removed)
                              FROM user_card_signal WHERE user_id = $1))",
    )
    .bind(user)
    .fetch_one(pool)
    .await
    .unwrap()
}

#[sqlx::test]
async fn the_seed_fills_every_activity_panel(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, uid) = app.register("seeded").await;
    app.verify_email(&uid).await;
    catalog(&pool).await;

    seed(&pool, "seeded").await.unwrap();

    let (status, body) = app.get(ACTIVITY_ROUTE, Some(&token)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let days = body["days"].as_array().unwrap().len();
    assert!(days > 180, "a busy year, with gaps: {days} active days");
    assert!(days < 365, "some days are quiet: {days} active days");
    for outcome in ["added", "skipped", "maybed", "removed"] {
        assert!(
            body["outcomes"][outcome].as_u64().unwrap() > 0,
            "{outcome}: {}",
            body["outcomes"]
        );
    }
    assert!(
        body["roles"].as_array().unwrap().len() >= 4,
        "{}",
        body["roles"]
    );
    assert!(
        !body["passes"].as_array().unwrap().is_empty(),
        "Most skipped has cards: {}",
        body["passes"]
    );
}

#[sqlx::test]
async fn the_seed_keeps_the_tables_in_step_and_replaces_on_a_rerun(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    let (_, uid) = app.register("rerun").await;
    let user = Uuid::parse_str(&uid).unwrap();
    catalog(&pool).await;

    seed(&pool, "rerun").await.unwrap();
    let first = totals(&pool, user).await;
    assert_eq!(first["lifetime"], first["daily"], "{first}");
    assert_eq!(first["daily"], first["weekly"], "{first}");
    assert_eq!(
        first["lifetime_searches"], first["daily_searches"],
        "{first}"
    );
    assert_eq!(first["weekly_outcomes"], first["card_outcomes"], "{first}");
    assert_eq!(first["daily"], first["weekly_outcomes"], "{first}");

    seed(&pool, "rerun").await.unwrap();
    let second = totals(&pool, user).await;
    assert_eq!(
        first, second,
        "a rerun replaces the year instead of adding a second one"
    );
}

#[sqlx::test]
async fn the_seed_refuses_an_unknown_account(pool: PgPool) {
    catalog(&pool).await;
    let err = seed(&pool, "nobody-here").await.unwrap_err();
    assert!(
        err.to_string().contains("no account named nobody-here"),
        "{err}"
    );
}

#[test]
fn the_wrapper_refuses_a_database_that_is_not_local() {
    let script = concat!(env!("CARGO_MANIFEST_DIR"), "/../zcripts/seed/activity.sh");
    for url in [
        "postgres://zwipe:secret@db.zwipe.net:5432/zwipe",
        "postgres://zwipe:secret@10.0.0.4/zwipe",
        "postgresql://zwipe@localhost/zwipe?host=db.zwipe.net",
    ] {
        let out = Command::new("bash")
            .arg(script)
            .arg("someone")
            .env("DATABASE_URL", url)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(1), "{url}");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(stderr.contains("refusing"), "{url}: {stderr}");
    }
}
