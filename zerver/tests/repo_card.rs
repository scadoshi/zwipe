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

use zwipe::{
    domain::card::ports::{CardRepository, DeckServeContext},
    outbound::sqlx::postgres::Postgres,
};
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
        .search_scryfall_data_deck_aware(
            &default_query(),
            DeckServeContext {
                synergy_scores: Some(&scores),
                ..Default::default()
            },
        )
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
            DeckServeContext {
                exclude_oracle_ids: &[drop_oracle],
                synergy_scores: Some(&scores),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let names: Vec<&str> = served.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names,
        vec!["Keeper"],
        "excluded oracle_id must not be served"
    );
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
        .search_scryfall_data_deck_aware(
            &q,
            DeckServeContext {
                deck_id: Some(deck),
                synergy_scores: Some(&scores),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let names: Vec<&str> = first.iter().map(|s| s.name.as_str()).collect();

    assert_eq!(
        first.len(),
        4,
        "all four cards served, none dropped: {names:?}"
    );
    assert!(
        names.contains(&"Null Oracle"),
        "NULL-oracle card must be served: {names:?}"
    );

    // deterministic for a fixed (deck, day) seed
    let second = repo
        .search_scryfall_data_deck_aware(
            &q,
            DeckServeContext {
                deck_id: Some(deck),
                synergy_scores: Some(&scores),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let names2: Vec<&str> = second.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, names2, "same deck seed must yield the same order");
}

/// Phase 4: a deck's selected oracle tags lift matching cards within the
/// synergy serve. Both cards are unscored (equal base), so the otag term is the
/// only differentiator — and with no selected otags the term is dormant and the
/// name tiebreak wins (the zero-regression / revert guarantee).
#[sqlx::test]
async fn deck_oracle_tags_lift_matching_cards(pool: sqlx::PgPool) {
    seed_cards(
        &pool,
        &[
            // Alphabetically first, so it leads on the name tiebreak absent otags.
            card("Aaa Plain").mono("R"),
            card("Zzz Removal").mono("R").oracle_tags(&["spot-removal"]),
        ],
    )
    .await;

    // Empty score map: both cards take the unscored anchor, so base + signal are
    // equal and the otag term alone decides the order.
    let scores = json!({});
    let repo = Postgres { pool: pool.clone() };

    // With the deck's selected otag, the matching card is lifted to the front,
    // overriding the alphabetical tiebreak.
    let served = repo
        .search_scryfall_data_deck_aware(
            &default_query(),
            DeckServeContext {
                synergy_scores: Some(&scores),
                deck_oracle_tags: &["spot-removal".to_string()],
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let names: Vec<&str> = served.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names,
        vec!["Zzz Removal", "Aaa Plain"],
        "otag-matching card should lead when the deck selected that otag"
    );

    // Zero-regression: no selected otags => the term is dormant and ordering
    // falls back to the name tiebreak (byte-identical to pre-Phase-4).
    let served = repo
        .search_scryfall_data_deck_aware(
            &default_query(),
            DeckServeContext {
                synergy_scores: Some(&scores),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let names: Vec<&str> = served.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names,
        vec!["Aaa Plain", "Zzz Removal"],
        "no selected otags => name order, no otag lift"
    );
}

/// Phase 4 (production path): the otag term must reorder within the *banded /
/// wildcard* serve (deck_id present), not just the pure-score path. 30 unscored
/// cards; four otag-matching cards are named to sort last, so absent otags they
/// land in band 1 (off the 24-card first page). With the deck's selected otag
/// the `W_ORACLE_TAG` lift pulls them into band 0 and onto the first page. This
/// exercises the wildcard CTE's `push_score` ranking (the path real serves use).
#[sqlx::test]
async fn deck_oracle_tags_lift_matching_cards_in_banded_serve(pool: sqlx::PgPool) {
    let mut fixtures: Vec<_> = (1..=26)
        .map(|i| card(&format!("Aaa{i:02}")).mono("R"))
        .collect();
    for i in 1..=4 {
        fixtures.push(
            card(&format!("Zzz{i:02}"))
                .mono("R")
                .oracle_tags(&["spot-removal"]),
        );
    }
    seed_cards(&pool, &fixtures).await;

    // Empty score map => every card takes the unscored anchor, so the otag term
    // is the only thing that moves a card between bands.
    let scores = json!({});
    let repo = Postgres { pool: pool.clone() };
    let deck = Uuid::from_u128(0xB4_0004);
    let q = default_query();

    let served_without = repo
        .search_scryfall_data_deck_aware(
            &q,
            DeckServeContext {
                deck_id: Some(deck),
                synergy_scores: Some(&scores),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let zzz_without = served_without
        .iter()
        .filter(|s| s.name.starts_with("Zzz"))
        .count();
    assert_eq!(
        zzz_without, 0,
        "absent otags, the otag cards sit in band 1, off the first page"
    );

    let served_with = repo
        .search_scryfall_data_deck_aware(
            &q,
            DeckServeContext {
                deck_id: Some(deck),
                synergy_scores: Some(&scores),
                deck_oracle_tags: &["spot-removal".to_string()],
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let zzz_with = served_with
        .iter()
        .filter(|s| s.name.starts_with("Zzz"))
        .count();
    assert!(
        zzz_with >= 1,
        "with the deck's selected otag, matching cards are lifted onto the first page (got {zzz_with})"
    );
}
