//! Card serving + search through the real router, backed by the fixture
//! builder in `common` (`card()` / `seed_cards()`). This proves the seeded
//! `scryfall_data` rows round-trip back through `DatabaseScryfallData` and that
//! the search filters behave over a known fixture set.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

mod common;

use axum::http::StatusCode;
use common::{TestApp, card, seed_cards};
use serde_json::json;

/// Reads the `name` off every card in a search result (a `Vec<Card>`).
fn names(results: &serde_json::Value) -> Vec<String> {
    results
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["scryfall_data"]["name"].as_str().unwrap().to_string())
        .collect()
}

#[sqlx::test]
async fn get_card_by_id_round_trips(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let goblin = card("Goblin Guide")
        .mono("R")
        .cmc(1.0)
        .mana_cost("{R}")
        .type_line("Creature — Goblin")
        .power("2")
        .toughness("2")
        .rarity("rare");
    let id = goblin.id();
    seed_cards(&pool, &[goblin]).await;

    // Public route, no token. If any NOT NULL column or JSONB shape were wrong
    // the read-path `try_from` would 500 here instead of 200.
    let (status, c) = app.get(&format!("/api/card/{id}"), None).await;
    assert_eq!(status, StatusCode::OK, "get card: {c}");
    let sd = &c["scryfall_data"];
    assert_eq!(sd["name"], "Goblin Guide");
    assert_eq!(sd["cmc"], 1.0);
    assert_eq!(sd["colors"], json!(["R"]));
    assert_eq!(sd["color_identity"], json!(["R"]));
    assert_eq!(sd["type_line"], "Creature — Goblin");
    assert_eq!(sd["rarity"], "rare");
}

#[sqlx::test]
async fn get_missing_card_is_404(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    seed_cards(&pool, &[]).await; // populate the (empty) view so queries don't error
    let missing = uuid::Uuid::from_u128(0xDEAD_BEEF);
    let (status, _) = app.get(&format!("/api/card/{missing}"), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn search_by_name_contains(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, _) = app.register("searcher").await;
    seed_cards(
        &pool,
        &[
            card("Goblin Guide").mono("R"),
            card("Llanowar Elves").mono("G"),
            card("Goblin Matron").mono("R"),
        ],
    )
    .await;

    let (status, results) = app
        .post("/api/card/search", json!({ "name_contains": "Goblin" }), Some(&token))
        .await;
    assert_eq!(status, StatusCode::OK, "search: {results}");
    let mut got = names(&results);
    got.sort();
    assert_eq!(got, vec!["Goblin Guide", "Goblin Matron"], "name filter should match only goblins");
}

#[sqlx::test]
async fn search_by_cmc_range(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, _) = app.register("ranger").await;
    seed_cards(
        &pool,
        &[
            card("One Drop").mono("R").cmc(1.0),
            card("Three Drop").mono("R").cmc(3.0),
            card("Six Drop").mono("R").cmc(6.0),
        ],
    )
    .await;

    let (status, results) = app
        .post("/api/card/search", json!({ "cmc_range": [2.0, 4.0] }), Some(&token))
        .await;
    assert_eq!(status, StatusCode::OK, "search: {results}");
    assert_eq!(names(&results), vec!["Three Drop"], "only the cmc-3 card is in [2,4]");
}

#[sqlx::test]
async fn search_color_identity_within(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, _) = app.register("colorist").await;
    seed_cards(
        &pool,
        &[
            card("Red Card").mono("R"),
            card("Green Card").mono("G"),
            card("Colorless Card").color_identity(""),
        ],
    )
    .await;

    // Empty color identity is a subset of {R}, so the colorless card rides along;
    // the green card is excluded.
    let (status, results) = app
        .post("/api/card/search", json!({ "color_identity_within": ["R"] }), Some(&token))
        .await;
    assert_eq!(status, StatusCode::OK, "search: {results}");
    let mut got = names(&results);
    got.sort();
    assert_eq!(got, vec!["Colorless Card", "Red Card"], "within-R must exclude the green card");
}

#[sqlx::test]
async fn search_requires_auth(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    seed_cards(&pool, &[card("Any Card").mono("R")]).await;
    let (status, _) = app.post("/api/card/search", json!({ "name_contains": "Any" }), None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
