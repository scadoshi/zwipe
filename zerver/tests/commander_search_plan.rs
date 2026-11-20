//! The commander search ranks a pool of ids and sort keys and joins the printings back for the
//! page it serves. Every page it serves holds the same cards in the same order as the statement
//! that carried whole printings through the ranking, kept here as the reference.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    reason = "a test: helpers outside #[test] fns unwrap and index, and a panic is the failure report"
)]

mod common;

use std::time::Duration;

use axum::http::StatusCode;
use chrono::Utc;
use common::{TestApp, card, seed_cards};
use serde_json::{Value, json};
use uuid::Uuid;
use zwipe::inbound::http::rate_limits::RateLimits;
use zwipe_core::http::paths::SEARCH_COMMANDERS_ROUTE;

/// The statement the commander search sent for the swipe stack's query before the pool held
/// keys only: every column of `latest_cards` through the window. `$1` is the shuffle seed, `$4`
/// to `$6` the band's limit and offset and the deep slice's offset.
const REFERENCE: &str = "WITH pool AS (SELECT latest_cards.*, row_number() OVER (ORDER BY pop.pop_decks DESC NULLS LAST, latest_cards.edhrec_rank ASC NULLS LAST, latest_cards.name ASC, latest_cards.id) AS rn, hashtext(COALESCE(latest_cards.oracle_id::text, '') || $1) AS shuffle, FALSE AS pool_returned, COALESCE(sel.sel_shown, 0) AS pool_shown
     FROM latest_cards
     JOIN card_profiles ON latest_cards.id = card_profiles.scryfall_data_id
     LEFT JOIN (SELECT oracle_id AS pop_oracle_id, decks AS pop_decks FROM commander_popularity) pop ON pop.pop_oracle_id = latest_cards.oracle_id
     LEFT JOIN (SELECT commander_oracle_id AS sel_oid, shown AS sel_shown FROM commander_select_signal) sel ON sel.sel_oid = latest_cards.oracle_id
     WHERE TRUE AND latest_cards.layout NOT IN ('token', 'double_faced_token', 'emblem') AND  card_profiles.is_token = $2 AND (legalities->>$3 IN ('legal', 'restricted')) AND NOT (latest_cards.layout = 'meld' AND EXISTS (SELECT 1 FROM jsonb_array_elements(latest_cards.all_parts) mp WHERE mp->>'component' = 'meld_result' AND mp->>'name' = latest_cards.name)) AND ((type_line ILIKE '%Legendary%' AND split_part(type_line, ' // ', 1) ILIKE '%Legendary%' AND ((type_line ILIKE '%Creature%' AND split_part(type_line, ' // ', 1) ILIKE '%Creature%') OR (power IS NOT NULL AND toughness IS NOT NULL))) OR oracle_text ILIKE '%can be your commander%')) SELECT * FROM ((SELECT *, 0 AS slice FROM pool ORDER BY pool_returned, (rn - 1) / 25, shuffle, name ASC, id LIMIT $4 OFFSET $5) UNION ALL (SELECT *, 1 AS slice FROM pool WHERE rn > 500 ORDER BY pool_returned, pool_shown ASC, shuffle, id LIMIT 1 OFFSET $6)) AS combined ORDER BY slice ASC, pool_returned, (rn - 1) / 25, shuffle, name ASC, id";

/// Cards a page holds.
const PAGE: u32 = 25;

/// Where the server lifts the deep slice's card to in a page.
const WILDCARD_POSITION: usize = 17;

/// More commanders than the deep slice's floor of 500, so pages carry its probe. Ranks and
/// popularity repeat, so the name and id tiebreaks decide too.
fn commander_pool() -> Vec<common::CardFixture> {
    let mut cards: Vec<common::CardFixture> = (0..620)
        .map(|i| {
            card(&format!("Plan Commander {i:03}"))
                .type_line("Legendary Creature — Goblin")
                .power("1")
                .toughness("1")
                .commander_legal()
                .edhrec_rank(i32::try_from(i / 3).unwrap())
        })
        .collect();
    // Commanders by their text, a non-legendary creature and a banned one, which the pool
    // leaves out.
    cards.push(
        card("Plan Planeswalker")
            .type_line("Legendary Planeswalker — Plan")
            .oracle_text("Plan Planeswalker can be your commander.")
            .commander_legal(),
    );
    cards.push(
        card("Plan Grunt")
            .type_line("Creature — Goblin")
            .commander_legal(),
    );
    cards.push(card("Plan Banned").type_line("Legendary Creature — Goblin"));
    cards
}

/// The page at `offset` as the reference statement orders it, with the server's splice of the
/// deep slice's card into the band and its one copy of each printing.
async fn reference_page(pool: &sqlx::PgPool, seed: &str, offset: u32) -> Vec<Uuid> {
    let band_limit = i64::from(PAGE) - 1;
    let page_index = i64::from(offset / PAGE);
    let mut ids: Vec<Uuid> = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
        "SELECT id FROM ({REFERENCE}) AS r"
    )))
    .bind(seed)
    .bind(false)
    .bind("commander")
    .bind(band_limit)
    .bind(page_index * band_limit)
    .bind(page_index)
    .fetch_all(pool)
    .await
    .unwrap();
    let band = usize::try_from(band_limit).unwrap();
    if ids.len() > band {
        for wildcard in ids.split_off(band) {
            let at = WILDCARD_POSITION.min(ids.len());
            ids.insert(at, wildcard);
        }
    }
    // Sleeving the page keys it by id, so a probe the band already holds (the pool's tail) is
    // served once, where it first stands.
    let mut seen = std::collections::HashSet::new();
    ids.retain(|id| seen.insert(*id));
    ids
}

#[sqlx::test]
async fn every_commander_page_matches_the_whole_row_ranking(pool: sqlx::PgPool) {
    let cards = commander_pool();
    seed_cards(&pool, &cards).await;
    // Popularity for every other commander, in tied pairs.
    for (i, c) in cards
        .iter()
        .take(620)
        .enumerate()
        .filter(|(i, _)| i % 2 == 0)
    {
        sqlx::query(
            "INSERT INTO commander_popularity (oracle_id, name, decks) VALUES ($1, $2, $3)",
        )
        .bind(c.oracle_id())
        .bind(c.name())
        .bind(i64::try_from(1000 - i / 4).unwrap())
        .execute(&pool)
        .await
        .unwrap();
    }
    // Every page is a search, more than one burst's worth: the buckets refill at once.
    let limits = RateLimits::default().with_every_period(Duration::from_millis(1));
    let app = TestApp::with_limits(pool.clone(), &limits);
    let (token, user) = app.register("planner").await;
    let seed = format!("{user}:{}", Utc::now().date_naive());

    let mut served = 0;
    for offset in (0..650).step_by(25) {
        let (status, body) = app
            .post(
                SEARCH_COMMANDERS_ROUTE,
                json!({
                    "is_commander_in_format": "commander",
                    "legalities_contains_all": ["commander"],
                    "is_token": false,
                    "limit": PAGE,
                    "offset": offset,
                }),
                Some(&token),
            )
            .await;
        assert_eq!(status, StatusCode::OK, "offset {offset}: {body}");
        let got: Vec<Uuid> = body
            .as_array()
            .unwrap()
            .iter()
            .map(|c: &Value| c["scryfall_data"]["id"].as_str().unwrap().parse().unwrap())
            .collect();
        let want = reference_page(&pool, &seed, u32::try_from(offset).unwrap()).await;
        assert_eq!(got, want, "offset {offset}: same cards in the same order");
        served += got.len();
    }
    assert!(served > 600, "the pages walk the whole pool: {served}");
}
