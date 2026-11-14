//! A usage flush takes the pooled signal rows' locks in key order, so two
//! flushes that overlap on a few rows wait on each other instead of
//! deadlocking. Each test holds the lowest key's row in an open transaction,
//! lets a flush queue behind it, then takes every other key: a flush that
//! locked any row out of order deadlocks against it.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "a test: helpers outside #[test] fns unwrap and index, and a panic is the failure report"
)]

mod common;

use std::time::Duration;

use axum::http::StatusCode;
use common::{TestApp, card, seed_cards};
use serde_json::{Value, json};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;
use zwipe_core::http::paths::{DECK_ROUTE, RECORD_USAGE_ROUTE};

/// Rows each test touches; with this many, a random order almost never
/// starts on the lowest key.
const KEYS: usize = 12;

/// Waits until some backend in this database is waiting on a row lock.
async fn wait_for_lock_waiter(pool: &PgPool) {
    for _ in 0..200 {
        let waiting: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM pg_stat_activity \
             WHERE datname = current_database() AND wait_event_type = 'Lock'",
        )
        .fetch_one(pool)
        .await
        .unwrap();
        if waiting > 0 {
            return;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    panic!("the flush never queued behind the held row");
}

async fn create_deck(app: &TestApp, token: &str, format: &str) -> Uuid {
    let (status, deck) = app
        .post(
            DECK_ROUTE,
            json!({ "name": "Lock order", "format": format }),
            Some(token),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "create: {deck}");
    Uuid::parse_str(deck["id"].as_str().unwrap()).unwrap()
}

fn usage(signals: &[Value]) -> Value {
    json!({
        "swipes_right": 0, "swipes_left": 0, "swipes_up": 0, "swipes_down": 0, "searches": 0,
        "signals": signals,
    })
}

async fn upsert_otag(tx: &mut Transaction<'_, Postgres>, context: &str, tag: &str) {
    sqlx::query(
        "INSERT INTO otag_context_signal \
             (context_key, oracle_tag, shown, added, skipped, maybed, removed, updated_at) \
         VALUES ($1, $2, 1, 0, 0, 0, 0, NOW()) \
         ON CONFLICT (context_key, oracle_tag) DO UPDATE SET \
             shown = otag_context_signal.shown + 1",
    )
    .bind(context)
    .bind(tag)
    .execute(&mut **tx)
    .await
    .unwrap();
}

async fn upsert_commander_card(tx: &mut Transaction<'_, Postgres>, commander: Uuid, card: Uuid) {
    sqlx::query(
        "INSERT INTO commander_card_signal \
             (commander_oracle_id, card_oracle_id, shown, added, skipped, maybed, removed, updated_at) \
         VALUES ($1, $2, 1, 0, 0, 0, 0, NOW()) \
         ON CONFLICT (commander_oracle_id, card_oracle_id) DO UPDATE SET \
             shown = commander_card_signal.shown + 1",
    )
    .bind(commander)
    .bind(card)
    .execute(&mut **tx)
    .await
    .unwrap();
}

#[sqlx::test]
async fn otag_context_rows_lock_in_key_order(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, _) = app.register("otags").await;

    let tags: Vec<String> = (0..KEYS).map(|i| format!("tag-{i:02}")).collect();
    let tag_refs: Vec<&str> = tags.iter().map(String::as_str).collect();
    let blaze = card("Fireblast").oracle_tags(&tag_refs).color_identity("R");
    let (blaze_id, blaze_oracle) = (blaze.id(), blaze.oracle_id().unwrap());
    seed_cards(&pool, &[blaze]).await;

    let deck_id = create_deck(&app, &token, "modern").await;
    sqlx::query(
        "INSERT INTO deck_cards (deck_id, scryfall_data_id, oracle_id, quantity, board) \
         VALUES ($1, $2, $3, 1, 'deck')",
    )
    .bind(deck_id)
    .bind(blaze_id)
    .bind(blaze_oracle)
    .execute(&pool)
    .await
    .unwrap();
    let context = "format_ci:modern:R";

    let mut held = pool.begin().await.unwrap();
    upsert_otag(&mut held, context, &tags[0]).await;

    let flush = app.post(
        RECORD_USAGE_ROUTE,
        usage(&[json!({
            "card_oracle_id": blaze_oracle.to_string(),
            "deck_id": deck_id.to_string(),
            "shown": 1, "added": 0, "skipped": 0, "maybed": 0, "removed": 0
        })]),
        Some(&token),
    );
    let other = async {
        wait_for_lock_waiter(&pool).await;
        for tag in &tags[1..] {
            upsert_otag(&mut held, context, tag).await;
        }
        held.commit().await.unwrap();
    };
    let ((status, body), ()) = tokio::join!(flush, other);
    assert_eq!(status, StatusCode::NO_CONTENT, "flush: {body}");

    let shown: Vec<i64> = sqlx::query_scalar(
        "SELECT shown FROM otag_context_signal WHERE context_key = $1 ORDER BY oracle_tag",
    )
    .bind(context)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(shown, vec![2; KEYS], "both writers credited every tag");
}

#[sqlx::test]
async fn commander_card_rows_lock_in_key_order(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, _) = app.register("cmdr").await;

    let atraxa = card("Atraxa, Praetors' Voice").color_identity("WUBG");
    let (atraxa_id, atraxa_oracle) = (atraxa.id(), atraxa.oracle_id().unwrap());
    let cards: Vec<_> = (0..KEYS).map(|i| card(&format!("Card {i:02}"))).collect();
    let mut oracles: Vec<Uuid> = cards.iter().map(|c| c.oracle_id().unwrap()).collect();
    oracles.sort_unstable();
    let mut all = cards;
    all.push(atraxa);
    seed_cards(&pool, &all).await;

    let deck_id = create_deck(&app, &token, "commander").await;
    sqlx::query("UPDATE decks SET commander_id = $1 WHERE id = $2")
        .bind(atraxa_id)
        .bind(deck_id)
        .execute(&pool)
        .await
        .unwrap();

    let mut held = pool.begin().await.unwrap();
    upsert_commander_card(&mut held, atraxa_oracle, oracles[0]).await;

    // The client sends its signals in descending key order.
    let signals: Vec<Value> = oracles
        .iter()
        .rev()
        .map(|oracle| {
            json!({
                "card_oracle_id": oracle.to_string(),
                "deck_id": deck_id.to_string(),
                "shown": 1, "added": 0, "skipped": 0, "maybed": 0, "removed": 0
            })
        })
        .collect();
    let flush = app.post(RECORD_USAGE_ROUTE, usage(&signals), Some(&token));
    let other = async {
        wait_for_lock_waiter(&pool).await;
        for oracle in &oracles[1..] {
            upsert_commander_card(&mut held, atraxa_oracle, *oracle).await;
        }
        held.commit().await.unwrap();
    };
    let ((status, body), ()) = tokio::join!(flush, other);
    assert_eq!(status, StatusCode::NO_CONTENT, "flush: {body}");

    let shown: Vec<i64> = sqlx::query_scalar(
        "SELECT shown FROM commander_card_signal WHERE commander_oracle_id = $1 \
         ORDER BY card_oracle_id",
    )
    .bind(atraxa_oracle)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(shown, vec![2; KEYS], "both writers credited every card");
}
