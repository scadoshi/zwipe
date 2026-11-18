//! The copy cap and the deck limit through the real router: a request sets at most 99 copies of
//! a card, a quantity patch counts toward the deck limit the way an add does, the command zone
//! counts toward it too, the database refuses a count outside 1 to 99, and a deck already past
//! its total loads, can go down and cannot go up.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "a test: helpers outside #[test] fns unwrap and index, and a panic is the failure report"
)]

mod common;

use axum::http::{Method, StatusCode};
use common::{TestApp, card, seed_cards};
use serde_json::{Value, json};
use uuid::Uuid;
use zwipe_core::http::messages::{CARD_LIMIT, UNVERIFIED_CARD_LIMIT};
use zwipe_core::http::paths::{
    DECK_ROUTE, create_deck_card_route, deck_route, import_deck_cards_route, update_deck_card_route,
};

/// What a request past the copy cap reads.
const OVER_99: &str = "invalid quantity: must be between 1 and 99";

/// A user's commander deck: `(token, deck_id)`. Verified when `verified`.
async fn deck_for(app: &TestApp, username: &str, verified: bool) -> (String, Uuid) {
    let (token, uid) = app.register(username).await;
    if verified {
        app.verify_email(&uid).await;
    }
    let (status, deck) = app
        .post(
            DECK_ROUTE,
            json!({ "name": "Limits", "format": "commander" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "deck create: {deck}");
    (token, deck["id"].as_str().unwrap().parse().unwrap())
}

/// Seeds cards named `Card 0` up: their `(scryfall_data_id, oracle_id)`.
async fn seed(pool: &sqlx::PgPool, count: usize) -> Vec<(Uuid, Uuid)> {
    let fixtures: Vec<_> = (0..count)
        .map(|n| card(&format!("Card {n}")).type_line("Creature"))
        .collect();
    let ids = fixtures
        .iter()
        .map(|c| (c.id(), c.oracle_id().unwrap()))
        .collect();
    seed_cards(pool, &fixtures).await;
    ids
}

async fn add(
    app: &TestApp,
    token: &str,
    deck_id: Uuid,
    (sid, oid): (Uuid, Uuid),
    quantity: i32,
) -> (StatusCode, Value) {
    app.send_text(
        Method::POST,
        &create_deck_card_route(deck_id),
        json!({ "scryfall_data_id": sid.to_string(), "oracle_id": oid.to_string(), "quantity": quantity }),
        token,
    )
    .await
}

async fn set_quantity(
    app: &TestApp,
    token: &str,
    deck_id: Uuid,
    sid: Uuid,
    quantity: i32,
) -> (StatusCode, Value) {
    app.send_text(
        Method::PATCH,
        &update_deck_card_route(deck_id, sid),
        json!({ "quantity": quantity }),
        token,
    )
    .await
}

/// Writes a count straight to the database, past the router's checks.
async fn store_quantity(
    pool: &sqlx::PgPool,
    deck_id: Uuid,
    sid: Uuid,
    quantity: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE deck_cards SET quantity = $1 WHERE deck_id = $2 AND scryfall_data_id = $3")
        .bind(quantity)
        .bind(deck_id)
        .bind(sid)
        .execute(pool)
        .await
        .map(|_| ())
}

#[sqlx::test]
async fn an_add_sets_at_most_99_copies(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, deck_id) = deck_for(&app, "addcap", true).await;
    let cards = seed(&pool, 2).await;

    let (status, body) = add(&app, &token, deck_id, cards[0], 100).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body, json!(OVER_99));
    let (status, body) = add(&app, &token, deck_id, cards[1], 99).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
}

#[sqlx::test]
async fn a_patch_raises_a_card_to_99_and_no_further(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, deck_id) = deck_for(&app, "patchcap", true).await;
    let cards = seed(&pool, 1).await;
    let (status, body) = add(&app, &token, deck_id, cards[0], 98).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");

    let (status, body) = set_quantity(&app, &token, deck_id, cards[0].0, 99).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = set_quantity(&app, &token, deck_id, cards[0].0, 100).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body, json!(OVER_99));
}

#[sqlx::test]
async fn the_database_holds_a_card_to_1_through_99_copies(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, deck_id) = deck_for(&app, "stored", true).await;
    let cards = seed(&pool, 1).await;
    let (status, body) = add(&app, &token, deck_id, cards[0], 1).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");

    assert!(
        store_quantity(&pool, deck_id, cards[0].0, 100)
            .await
            .is_err()
    );
    assert!(store_quantity(&pool, deck_id, cards[0].0, 0).await.is_err());
    assert!(store_quantity(&pool, deck_id, cards[0].0, 99).await.is_ok());
}

#[sqlx::test]
async fn a_patch_counts_toward_the_unverified_deck_limit(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, deck_id) = deck_for(&app, "unverified", false).await;
    let cards = seed(&pool, 2).await;
    for (card, quantity) in [(cards[0], 60), (cards[1], 40)] {
        let (status, body) = add(&app, &token, deck_id, card, quantity).await;
        assert_eq!(status, StatusCode::CREATED, "{body}");
    }

    let (status, body) = set_quantity(&app, &token, deck_id, cards[1].0, 41).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body, json!(UNVERIFIED_CARD_LIMIT));
    let (status, body) = set_quantity(&app, &token, deck_id, cards[1].0, 39).await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

#[sqlx::test]
async fn a_patch_counts_toward_the_verified_deck_limit(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, deck_id) = deck_for(&app, "verified", true).await;
    let cards = seed(&pool, 6).await;
    for (n, card) in cards.iter().enumerate() {
        let quantity = if n < 5 { 99 } else { 5 };
        let (status, body) = add(&app, &token, deck_id, *card, quantity).await;
        assert_eq!(status, StatusCode::CREATED, "{body}");
    }

    let (status, body) = set_quantity(&app, &token, deck_id, cards[5].0, 6).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body, json!(CARD_LIMIT));
    let (status, body) = set_quantity(&app, &token, deck_id, cards[5].0, 5).await;
    assert_eq!(status, StatusCode::OK, "an unchanged count passes: {body}");
}

#[sqlx::test]
async fn a_deck_past_its_limit_loads_and_only_goes_down(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, deck_id) = deck_for(&app, "overdeck", false).await;
    let cards = seed(&pool, 3).await;
    for card in &cards[..2] {
        let (status, body) = add(&app, &token, deck_id, *card, 50).await;
        assert_eq!(status, StatusCode::CREATED, "{body}");
    }
    store_quantity(&pool, deck_id, cards[1].0, 70)
        .await
        .unwrap();

    let (status, deck) = app.get(&deck_route(deck_id), Some(&token)).await;
    assert_eq!(status, StatusCode::OK, "the deck loads: {deck}");
    let (status, body) = set_quantity(&app, &token, deck_id, cards[0].0, 51).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body, json!(UNVERIFIED_CARD_LIMIT));
    let (status, body) = add(&app, &token, deck_id, cards[2], 1).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    let (status, body) = set_quantity(&app, &token, deck_id, cards[0].0, 49).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "lowering passes though the deck stays over: {body}"
    );
}

#[sqlx::test]
async fn the_commander_counts_toward_the_deck_limit_and_may_join_a_full_deck(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, deck_id) = deck_for(&app, "zonelimit", false).await;
    let commander = card("Limit Commander")
        .mono("G")
        .type_line("Legendary Creature — Elf")
        .commander_legal();
    let commander_id = commander.id();
    seed_cards(&pool, &[commander]).await;
    let cards = seed(&pool, 3).await;
    for (card, quantity) in [(cards[0], 60), (cards[1], 40)] {
        let (status, body) = add(&app, &token, deck_id, card, quantity).await;
        assert_eq!(status, StatusCode::CREATED, "{body}");
    }

    let (status, body) = app
        .patch(
            &deck_route(deck_id),
            json!({ "commander_id": commander_id.to_string() }),
            Some(&token),
        )
        .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "a commander joins a full deck: {body}"
    );

    let (status, body) = set_quantity(&app, &token, deck_id, cards[1].0, 39).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = add(&app, &token, deck_id, cards[2], 1).await;
    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "99 cards and the commander fill it: {body}"
    );
    assert_eq!(body, json!(UNVERIFIED_CARD_LIMIT));
    let (status, body) = set_quantity(&app, &token, deck_id, cards[1].0, 40).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
}

#[sqlx::test]
async fn an_import_line_above_99_is_left_unresolved(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, deck_id) = deck_for(&app, "importcap", true).await;
    seed(&pool, 2).await;

    let (status, body) = app
        .post(
            &import_deck_cards_route(deck_id),
            json!({ "text": "100 Card 0\n60 Card 1\n40 Card 1" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let unresolved: Vec<(&str, &str)> = body["unresolved"]
        .as_array()
        .unwrap()
        .iter()
        .map(|u| (u["name"].as_str().unwrap(), u["reason"].as_str().unwrap()))
        .collect();
    assert_eq!(
        unresolved,
        [
            ("Card 0", "quantity too large"),
            ("Card 1", "quantity too large")
        ]
    );
    let (_, deck) = app.get(&deck_route(deck_id), Some(&token)).await;
    assert_eq!(deck["entries"], json!([]), "nothing lands: {deck}");
}
