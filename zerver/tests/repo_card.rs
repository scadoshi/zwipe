//! Repo-level card tests: the SQL whose logic doesn't surface cleanly through
//! HTTP. Constructs `Postgres { pool }` and calls `CardRepository` methods
//! directly (no router). Covers the default synergy ordering, the
//! `card_signal_rollup` math, and the deck-aware serve's NULL-`oracle_id`
//! handling (the 2026-07-06 regression: `NULL || seed` NULLed the shuffle key).
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

mod common;

use common::{card, refresh_card_views, seed_cards};
use serde_json::json;
use uuid::Uuid;

use zwipe::domain::card::ports::CardRepository;
use zwipe::outbound::sqlx::postgres::Postgres;
use zwipe_core::domain::card::search_card::card_filter::CardQuery;

/// A default `CardQuery` — no criteria, no explicit sort (so the synergy /
/// popularity ordering is the one under test).
fn default_query() -> CardQuery {
    serde_json::from_value(json!({})).unwrap()
}

async fn insert_signal(
    pool: &sqlx::PgPool,
    commander: Uuid,
    card_oracle: Uuid,
    shown: i64,
    added: i64,
    maybed: i64,
    removed: i64,
) {
    sqlx::query(
        "INSERT INTO commander_card_signal \
         (commander_oracle_id, card_oracle_id, shown, added, maybed, removed) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(commander)
    .bind(card_oracle)
    .bind(shown)
    .bind(added)
    .bind(maybed)
    .bind(removed)
    .execute(pool)
    .await
    .unwrap();
}

/// The rollup collapses every commander's rows for a card into one net/shown
/// pair: `net = Σ(added + 0.5·maybed − removed)`, `shown = Σ shown`.
#[sqlx::test]
async fn card_signal_rollup_math(pool: sqlx::PgPool) {
    let card_oracle = Uuid::from_u128(0xCA2D);
    let c1 = Uuid::from_u128(0xC001);
    let c2 = Uuid::from_u128(0xC002);

    // c1: 4 + 0.5*2 - 1 = 4.0 over 10 shown; c2: 1 + 0 - 0 = 1.0 over 5 shown.
    insert_signal(&pool, c1, card_oracle, 10, 4, 2, 1).await;
    insert_signal(&pool, c2, card_oracle, 5, 1, 0, 0).await;
    // a different card, to prove the GROUP BY isolates rows
    insert_signal(&pool, c1, Uuid::from_u128(0xBEEF), 3, 3, 0, 0).await;

    refresh_card_views(&pool).await;

    let (net, shown): (f64, f64) =
        sqlx::query_as("SELECT net, shown FROM card_signal_rollup WHERE card_oracle_id = $1")
            .bind(card_oracle)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(net, 5.0, "net = (4 + 1 - 1) + (1)");
    assert_eq!(shown, 15.0, "shown = 10 + 5");

    // the isolated card carries only its own row
    let (net2, shown2): (f64, f64) =
        sqlx::query_as("SELECT net, shown FROM card_signal_rollup WHERE card_oracle_id = $1")
            .bind(Uuid::from_u128(0xBEEF))
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!((net2, shown2), (3.0, 3.0));
}

/// With no deck seed, the deck-aware serve is pure score order: scored cards by
/// synergy score descending, unscored cards anchored below the floor
/// (UNSCORED_ANCHOR). No banding, no shuffle — deterministic.
#[sqlx::test]
async fn synergy_ordering_scored_before_unscored(pool: sqlx::PgPool) {
    seed_cards(
        &pool,
        &[
            card("Alpha Card").mono("R"),
            card("Bravo Card").mono("R"),
            card("Zeta Card").mono("R"), // unscored
        ],
    )
    .await;

    // keys are LOWER(name); Zeta is absent so it takes the anchor.
    let scores = json!({ "alpha card": 5.0, "bravo card": 1.0 });
    let repo = Postgres { pool: pool.clone() };

    let served = repo
        .search_scryfall_data_deck_aware(&default_query(), None, &[], Some(&scores), false, None)
        .await
        .unwrap();
    let names: Vec<&str> = served.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, vec!["Alpha Card", "Bravo Card", "Zeta Card"]);
}

/// `exclude_oracle_ids` drops a card from the deck-aware serve (the deck's own
/// cards must not be re-served).
#[sqlx::test]
async fn deck_aware_serve_excludes_oracle_ids(pool: sqlx::PgPool) {
    let keep = card("Keeper").mono("R");
    let drop = card("Dropped").mono("R");
    let drop_oracle = drop.oracle_id().unwrap();
    seed_cards(&pool, &[keep, drop]).await;

    let scores = json!({});
    let repo = Postgres { pool: pool.clone() };
    let served = repo
        .search_scryfall_data_deck_aware(
            &default_query(),
            None,
            &[drop_oracle],
            Some(&scores),
            false,
            None,
        )
        .await
        .unwrap();
    let names: Vec<&str> = served.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, vec!["Keeper"], "excluded oracle_id must not be served");
}

/// Regression (2026-07-06): a card with a NULL `oracle_id` must survive the
/// deck-seeded banded shuffle. The shuffle key is
/// `hashtext(COALESCE(oracle_id::text, '') || seed)`; before the COALESCE,
/// `NULL || seed` NULLed the key and mis-ordered/hid these cards. Here we assert
/// the NULL-oracle card is still served, the whole set comes back, and the
/// order is deterministic for a fixed deck seed.
#[sqlx::test]
async fn null_oracle_card_survives_deck_aware_shuffle(pool: sqlx::PgPool) {
    seed_cards(
        &pool,
        &[
            card("Real One").mono("R"),
            card("Null Oracle").mono("R").oracle(None),
            card("Real Two").mono("R"),
            card("Real Three").mono("R"),
        ],
    )
    .await;

    let scores = json!({});
    let deck = Uuid::from_u128(0xDEC0);
    let repo = Postgres { pool: pool.clone() };
    let q = default_query();

    let first = repo
        .search_scryfall_data_deck_aware(&q, Some(deck), &[], Some(&scores), false, None)
        .await
        .unwrap();
    let names: Vec<&str> = first.iter().map(|s| s.name.as_str()).collect();

    assert_eq!(first.len(), 4, "all four cards served, none dropped: {names:?}");
    assert!(names.contains(&"Null Oracle"), "NULL-oracle card must be served: {names:?}");

    // deterministic for a fixed (deck, day) seed
    let second = repo
        .search_scryfall_data_deck_aware(&q, Some(deck), &[], Some(&scores), false, None)
        .await
        .unwrap();
    let names2: Vec<&str> = second.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, names2, "same deck seed must yield the same order");
}
