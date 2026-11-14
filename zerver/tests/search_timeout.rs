//! Card search's statement timeout: Postgres cancels a search that runs past
//! the adapter's `search_timeout`, and the setting stays inside that search.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![expect(
    clippy::unwrap_used,
    reason = "a test: helpers outside #[test] fns unwrap, and a panic is the failure report"
)]

mod common;

use std::time::Duration;

use common::{card, seed_cards};
use zwipe::domain::card::ports::{CardRepository, DeckServeContext};
use zwipe::outbound::sqlx::postgres::Postgres;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;

/// A name filter long enough that matching it against every seeded row takes
/// far longer than the test's timeout. The builder path has no length cap,
/// so the adapter sees the full value.
fn slow_query() -> zwipe_core::domain::card::search_card::card_filter::CardQuery {
    CardQueryBuilder::new()
        .set_name_contains("a".repeat(4_000_000))
        .build()
        .unwrap()
}

#[sqlx::test]
async fn search_past_the_timeout_is_cancelled(pool: sqlx::PgPool) {
    let cards: Vec<_> = (0..40).map(|i| card(&format!("Card {i}"))).collect();
    seed_cards(&pool, &cards).await;
    let repo = Postgres::from_pool(pool.clone()).with_search_timeout(Duration::from_millis(20));

    let result = repo
        .search_scryfall_data_deck_aware(&slow_query(), DeckServeContext::default())
        .await;
    assert!(result.is_err(), "the slow search ran to completion");

    let (timeout,): (String,) = sqlx::query_as("SHOW statement_timeout")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(timeout, "0", "the search's timeout leaked onto the pool");
}

#[sqlx::test]
async fn search_under_the_timeout_returns_rows(pool: sqlx::PgPool) {
    seed_cards(&pool, &[card("Krenko, Mob Boss"), card("Goblin Guide")]).await;
    let repo = Postgres::from_pool(pool.clone());
    let query = CardQueryBuilder::new()
        .set_name_contains("Krenko")
        .build()
        .unwrap();

    let found = repo
        .search_scryfall_data_deck_aware(&query, DeckServeContext::default())
        .await
        .unwrap();
    let names: Vec<&str> = found.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, vec!["Krenko, Mob Boss"]);
}
