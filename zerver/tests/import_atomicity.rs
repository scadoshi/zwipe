//! Import atomicity: ownership + card-limit check + upsert + replace reconcile
//! run in ONE transaction under a deck row lock (`FOR UPDATE`), so a failed
//! import leaves the deck exactly as it was and concurrent imports can't race
//! past the card limit (the old count-then-insert TOCTOU).
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

mod common;

use axum::http::StatusCode;
use common::{TestApp, card, seed_cards};
use serde_json::json;

/// Creates a verified user + a commander deck, returns `(token, deck_id)`.
async fn deck_for(app: &TestApp, username: &str) -> (String, String) {
    let (token, uid) = app.register(username).await;
    app.verify_email(&uid).await;
    let (status, deck) = app
        .post(
            "/api/deck",
            json!({ "name": "Atomic Deck", "format": "commander" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "deck create: {deck}");
    (token, deck["id"].as_str().unwrap().to_string())
}

/// Sum of all card quantities in the deck, straight from the DB.
async fn total_quantity(pool: &sqlx::PgPool, deck_id: &str) -> i64 {
    let did: uuid::Uuid = deck_id.parse().unwrap();
    sqlx::query_scalar::<_, Option<i64>>("SELECT SUM(quantity) FROM deck_cards WHERE deck_id = $1")
        .bind(did)
        .fetch_one(pool)
        .await
        .unwrap()
        .unwrap_or(0)
}

#[sqlx::test]
async fn over_limit_import_writes_nothing(pool: sqlx::PgPool) {
    // The verified cap is 500 (all boards). An over-limit REPLACE import must
    // roll back entirely: no new cards land AND the pre-existing card survives
    // (before this fix the upsert committed before the limit-triggering state
    // could be reconciled, stranding a hybrid board on failure).
    let app = TestApp::new(pool.clone());
    let (token, did) = deck_for(&app, "overlimit").await;
    seed_cards(
        &pool,
        &[
            card("Lightning Bolt").mono("R").type_line("Instant"),
            card("Island").type_line("Basic Land — Island"),
        ],
    )
    .await;

    let (status, r) = app
        .post(
            &format!("/api/deck/{did}/card/import"),
            json!({ "text": "1 Lightning Bolt" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "seed import: {r}");

    let (status, r) = app
        .post(
            &format!("/api/deck/{did}/card/import"),
            json!({ "text": "600 Island", "mode": "replace" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "limit: {r}");

    // Nothing changed: Bolt still there, no Islands, total untouched.
    let (_, full) = app.get(&format!("/api/deck/{did}"), Some(&token)).await;
    let names: Vec<&str> = full["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["card"]["scryfall_data"]["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        vec!["Lightning Bolt"],
        "rollback left the deck as-is"
    );
    assert_eq!(total_quantity(&pool, &did).await, 1);
}

#[sqlx::test]
async fn replace_reconciles_only_boards_present_in_the_import(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, did) = deck_for(&app, "replacer").await;
    seed_cards(
        &pool,
        &[
            card("Lightning Bolt").mono("R").type_line("Instant"),
            card("Shock").mono("R").type_line("Instant"),
            card("Opt").mono("U").type_line("Instant"),
        ],
    )
    .await;

    let (status, r) = app
        .post(
            &format!("/api/deck/{did}/card/import"),
            json!({ "text": "1 Lightning Bolt\n1 Shock" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "seed import: {r}");

    // Replace the deck board with Opt only: Bolt + Shock must go, Opt stays.
    let (status, r) = app
        .post(
            &format!("/api/deck/{did}/card/import"),
            json!({ "text": "1 Opt", "mode": "replace" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "replace import: {r}");

    let (_, full) = app.get(&format!("/api/deck/{did}"), Some(&token)).await;
    let names: Vec<&str> = full["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["card"]["scryfall_data"]["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        vec!["Opt"],
        "deck board is exactly the imported list"
    );

    // An import where nothing resolves must not wipe anything.
    let (status, r) = app
        .post(
            &format!("/api/deck/{did}/card/import"),
            json!({ "text": "1 Definitely Not A Real Card", "mode": "replace" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "empty replace: {r}");
    assert_eq!(
        total_quantity(&pool, &did).await,
        1,
        "nothing-resolved replace left the board alone"
    );
}

#[sqlx::test]
async fn add_mode_overlap_replaces_quantities(pool: sqlx::PgPool) {
    // Re-importing an oracle_id replaces its quantity (upsert), and the limit
    // math subtracts the overlap: 2 Bolt then 3 Bolt = 3 Bolt, not 5.
    let app = TestApp::new(pool.clone());
    let (token, did) = deck_for(&app, "overlapper").await;
    seed_cards(
        &pool,
        &[card("Lightning Bolt").mono("R").type_line("Instant")],
    )
    .await;

    for qty in [2, 3] {
        let (status, r) = app
            .post(
                &format!("/api/deck/{did}/card/import"),
                json!({ "text": format!("{qty} Lightning Bolt") }),
                Some(&token),
            )
            .await;
        assert_eq!(status, StatusCode::OK, "import {qty}: {r}");
    }
    assert_eq!(
        total_quantity(&pool, &did).await,
        3,
        "second import replaced the quantity"
    );
}

#[sqlx::test]
async fn foreign_deck_import_is_rejected_and_writes_nothing(pool: sqlx::PgPool) {
    // Over HTTP a foreign deck 404s at the importer's get_deck_profile
    // pre-check (existence-hiding). The repo-level FOR UPDATE ownership check
    // (Forbidden) sits behind it as defense-in-depth for any future caller
    // that skips the pre-check.
    let app = TestApp::new(pool.clone());
    let (_owner_token, did) = deck_for(&app, "importowner").await;
    let (attacker_token, _) = app.register("importattacker").await;
    seed_cards(
        &pool,
        &[card("Lightning Bolt").mono("R").type_line("Instant")],
    )
    .await;

    for text in ["1 Lightning Bolt", "1 Definitely Not A Real Card"] {
        let (status, _) = app
            .post(
                &format!("/api/deck/{did}/card/import"),
                json!({ "text": text }),
                Some(&attacker_token),
            )
            .await;
        assert_eq!(status, StatusCode::NOT_FOUND, "foreign deck hidden: {text}");
    }
    assert_eq!(total_quantity(&pool, &did).await, 0);
}

#[sqlx::test]
async fn concurrent_imports_cannot_race_past_the_limit(pool: sqlx::PgPool) {
    // Two imports, each under the 500 cap alone (300 + 300), jointly over it.
    // The FOR UPDATE deck lock serializes them: exactly one succeeds, and the
    // deck never exceeds the cap. Before this fix both could pass the pool-side
    // count and land 600 cards.
    let app = TestApp::new(pool.clone());
    let (token, did) = deck_for(&app, "racer").await;
    seed_cards(
        &pool,
        &[
            card("Island").type_line("Basic Land — Island"),
            card("Mountain").type_line("Basic Land — Mountain"),
        ],
    )
    .await;

    let path = format!("/api/deck/{did}/card/import");
    let a = app.post(&path, json!({ "text": "300 Island" }), Some(&token));
    let b = app.post(&path, json!({ "text": "300 Mountain" }), Some(&token));
    let ((status_a, ra), (status_b, rb)) = tokio::join!(a, b);

    let oks = [status_a, status_b]
        .iter()
        .filter(|s| **s == StatusCode::OK)
        .count();
    let limits = [status_a, status_b]
        .iter()
        .filter(|s| **s == StatusCode::UNPROCESSABLE_ENTITY)
        .count();
    assert_eq!(
        (oks, limits),
        (1, 1),
        "one import lands, one hits the limit: {ra} / {rb}"
    );
    assert_eq!(
        total_quantity(&pool, &did).await,
        300,
        "the deck never exceeds the cap"
    );
}
