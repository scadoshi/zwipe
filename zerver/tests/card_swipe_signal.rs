//! The pooled per-card swipe signal, `card_swipe_signal`: every deck the user owns feeds it,
//! with or without a commander, under the same daily budget as the commander-keyed pool, which
//! keeps its own writes unchanged. Its migration carries the commander-keyed history over, summed
//! per card.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![expect(
    clippy::unwrap_used,
    reason = "a test: helpers outside #[test] fns unwrap, and a panic is the failure report"
)]

use axum::http::StatusCode;
use common::{TestApp, card, seed_cards};
use serde_json::{Value, json};
use uuid::Uuid;
use zwipe_core::http::paths::{DECK_ROUTE, RECORD_USAGE_ROUTE};

mod common;

/// The migration before the one that creates `card_swipe_signal`.
const BEFORE_CARD_SWIPE_SIGNAL: i64 = 20_261_118_180_000;

/// A deck named `name` of `format` for the signed-in user, with `commander` set when given.
async fn deck(
    app: &TestApp,
    pool: &sqlx::PgPool,
    token: &str,
    (name, format): (&str, &str),
    commander: Option<Uuid>,
) -> Uuid {
    let (status, deck) = app
        .post(
            DECK_ROUTE,
            json!({ "name": name, "format": format }),
            Some(token),
        )
        .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "create {name} {format}: {deck}"
    );
    let deck_id = Uuid::parse_str(deck.get("id").and_then(Value::as_str).unwrap()).unwrap();
    if let Some(commander) = commander {
        sqlx::query("UPDATE decks SET commander_id = $1 WHERE id = $2")
            .bind(commander)
            .bind(deck_id)
            .execute(pool)
            .await
            .unwrap();
    }
    deck_id
}

/// One Add-stack signal for `card` swiped in `deck`, or in no deck.
fn signal(card: Uuid, deck: Option<Uuid>, shown: i64, added: i64, skipped: i64) -> Value {
    json!({
        "card_oracle_id": card.to_string(),
        "deck_id": deck.map(|d| d.to_string()),
        "shown": shown, "added": added, "skipped": skipped, "maybed": 0, "removed": 0
    })
}

/// Sends a usage flush holding `signals`.
async fn flush(app: &TestApp, token: &str, signals: Vec<Value>) {
    let (status, body) = app
        .post(
            RECORD_USAGE_ROUTE,
            json!({
                "swipes_right": 0, "swipes_left": 0, "swipes_up": 0, "swipes_down": 0,
                "searches": 0, "signals": signals
            }),
            Some(token),
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT, "usage flush: {body}");
}

/// The card's pooled swipe counts: shown, added, skipped; `None` with no row.
async fn swipes(pool: &sqlx::PgPool, card: Uuid) -> Option<(i64, i64, i64)> {
    sqlx::query_as("SELECT shown, added, skipped FROM card_swipe_signal WHERE card_oracle_id = $1")
        .bind(card)
        .fetch_optional(pool)
        .await
        .unwrap()
}

#[sqlx::test]
async fn a_swipe_in_a_deck_with_no_commander_reaches_the_card_s_row(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, _) = app.register("sixtycards").await;
    let bolt = card("Lightning Bolt");
    let bolt_oracle = bolt.oracle_id().unwrap();
    seed_cards(&pool, &[bolt]).await;
    let modern = deck(&app, &pool, &token, ("Burn", "modern"), None).await;

    flush(
        &app,
        &token,
        vec![signal(bolt_oracle, Some(modern), 3, 2, 1)],
    )
    .await;

    assert_eq!(swipes(&pool, bolt_oracle).await, Some((3, 2, 1)));
    let commander_rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM commander_card_signal")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        commander_rows, 0,
        "the commander-keyed pool still skips a deck with no commander"
    );
}

#[sqlx::test]
async fn swipes_of_one_card_in_every_deck_sum_to_one_row(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, user_id) = app.register("manydecks").await;
    app.verify_email(&user_id).await;
    let atraxa = card("Atraxa, Praetors' Voice");
    let atraxa_id = atraxa.id();
    let atraxa_oracle = atraxa.oracle_id().unwrap();
    let ring = card("Sol Ring");
    let ring_oracle = ring.oracle_id().unwrap();
    seed_cards(&pool, &[atraxa, ring]).await;
    let edh = deck(
        &app,
        &pool,
        &token,
        ("Atraxa", "commander"),
        Some(atraxa_id),
    )
    .await;
    let unled = deck(&app, &pool, &token, ("Unled", "commander"), None).await;

    flush(
        &app,
        &token,
        vec![
            signal(ring_oracle, Some(edh), 2, 1, 1),
            signal(ring_oracle, Some(unled), 1, 1, 0),
        ],
    )
    .await;
    assert_eq!(swipes(&pool, ring_oracle).await, Some((3, 2, 1)));

    flush(
        &app,
        &token,
        vec![signal(ring_oracle, Some(unled), 4, 0, 4)],
    )
    .await;
    assert_eq!(
        swipes(&pool, ring_oracle).await,
        Some((7, 2, 5)),
        "a later flush adds to the row"
    );

    let (shown, added): (i64, i64) = sqlx::query_as(
        "SELECT shown, added FROM commander_card_signal \
         WHERE commander_oracle_id = $1 AND card_oracle_id = $2",
    )
    .bind(atraxa_oracle)
    .bind(ring_oracle)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        (shown, added),
        (2, 1),
        "the commander-keyed pool keeps only its commander's deck"
    );
}

#[sqlx::test]
async fn only_known_cards_swiped_in_the_user_s_own_decks_count(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, _) = app.register("honest").await;
    let (other_token, _) = app.register("someoneelse").await;
    let bolt = card("Lightning Bolt");
    let bolt_oracle = bolt.oracle_id().unwrap();
    seed_cards(&pool, &[bolt]).await;
    let theirs = deck(&app, &pool, &other_token, ("Burn", "modern"), None).await;

    flush(
        &app,
        &token,
        vec![
            signal(bolt_oracle, Some(theirs), 5, 5, 0),
            signal(bolt_oracle, None, 5, 5, 0),
            signal(Uuid::from_u128(0xdead), Some(theirs), 5, 5, 0),
        ],
    )
    .await;

    let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM card_swipe_signal")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        rows, 0,
        "another user's deck, no deck and an unknown card add nothing"
    );
}

#[sqlx::test]
async fn a_flush_over_the_daily_budget_adds_nothing(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, _) = app.register("firehose").await;
    let bolt = card("Lightning Bolt");
    let bolt_oracle = bolt.oracle_id().unwrap();
    seed_cards(&pool, &[bolt]).await;
    let modern = deck(&app, &pool, &token, ("Burn", "modern"), None).await;
    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE username = 'firehose'")
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO usage_signal_budget (user_id, day, shown) \
         VALUES ($1, (NOW() AT TIME ZONE 'UTC')::date, 5000)",
    )
    .bind(user_id)
    .execute(&pool)
    .await
    .unwrap();

    flush(
        &app,
        &token,
        vec![signal(bolt_oracle, Some(modern), 1, 1, 0)],
    )
    .await;

    assert_eq!(swipes(&pool, bolt_oracle).await, None);
}

#[sqlx::test(migrations = false)]
async fn the_migration_carries_the_commander_history_over_per_card(pool: sqlx::PgPool) {
    let migrator = sqlx::migrate!("./migrations");
    migrator
        .run_to(BEFORE_CARD_SWIPE_SIGNAL, &pool)
        .await
        .unwrap();
    let (atraxa, edgar) = (Uuid::from_u128(1), Uuid::from_u128(2));
    let (ring, bolt) = (Uuid::from_u128(10), Uuid::from_u128(11));
    for (commander, card, shown, added, skipped, maybed, removed) in [
        (atraxa, ring, 10_i64, 7_i64, 2_i64, 1_i64, 1_i64),
        (edgar, ring, 4, 1, 3, 0, 0),
        (edgar, bolt, 2, 0, 2, 0, 0),
    ] {
        sqlx::query(
            "INSERT INTO commander_card_signal \
             (commander_oracle_id, card_oracle_id, shown, added, skipped, maybed, removed) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(commander)
        .bind(card)
        .bind(shown)
        .bind(added)
        .bind(skipped)
        .bind(maybed)
        .bind(removed)
        .execute(&pool)
        .await
        .unwrap();
    }

    migrator.run(&pool).await.unwrap();

    let rows: Vec<(Uuid, i64, i64, i64, i64, i64)> = sqlx::query_as(
        "SELECT card_oracle_id, shown, added, skipped, maybed, removed \
         FROM card_swipe_signal ORDER BY card_oracle_id",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(rows, [(ring, 14, 8, 5, 1, 1), (bolt, 2, 0, 2, 0, 0)]);
}
