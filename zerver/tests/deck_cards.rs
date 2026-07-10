//! Deck-card operations through the real router: add, quantity delta, board
//! placement, remove, and text import (resolved + unresolved). These were
//! deferred out of `deck_flows.rs` because they need real `cards` rows — the
//! `card()` / `seed_cards()` fixture builder now supplies them.
//!
//! Note the create route is `POST /api/deck/{id}/card` — no trailing slash
//! (the nested `/` leaf resolves without one, same as `/api/deck`).
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
        .post("/api/deck", json!({ "name": "Test Deck", "format": "commander" }), Some(&token))
        .await;
    assert_eq!(status, StatusCode::CREATED, "deck create: {deck}");
    (token, deck["id"].as_str().unwrap().to_string())
}

#[sqlx::test]
async fn deck_card_add_bump_remove(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, did) = deck_for(&app, "builder").await;

    let bolt = card("Lightning Bolt").mono("R").cmc(1.0).type_line("Instant");
    let sid = bolt.id();
    let oid = bolt.oracle_id().unwrap();
    seed_cards(&pool, &[bolt]).await;

    // add one copy
    let (status, dc) = app
        .post(
            &format!("/api/deck/{did}/card"),
            json!({ "scryfall_data_id": sid.to_string(), "oracle_id": oid.to_string(), "quantity": 1 }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "add card: {dc}");
    assert_eq!(dc["quantity"], 1);
    assert_eq!(dc["board"], "deck");

    // update_quantity is a delta: +2 => 3
    let (status, dc) = app
        .put(&format!("/api/deck/{did}/card/{sid}"), json!({ "update_quantity": 2 }), Some(&token))
        .await;
    assert_eq!(status, StatusCode::OK, "bump qty: {dc}");
    assert_eq!(dc["quantity"], 3);

    // the full deck reflects the card + quantity
    let (_, full) = app.get(&format!("/api/deck/{did}"), Some(&token)).await;
    let entries = full["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 1, "one entry expected: {full}");
    assert_eq!(entries[0]["card"]["scryfall_data"]["name"], "Lightning Bolt");
    assert_eq!(entries[0]["deck_card"]["quantity"], 3);

    // remove => 204, deck empties
    let (status, _) = app.delete(&format!("/api/deck/{did}/card/{sid}"), Some(&token)).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "delete card");
    let (_, full) = app.get(&format!("/api/deck/{did}"), Some(&token)).await;
    assert_eq!(full["entries"].as_array().unwrap().len(), 0, "deck should be empty after remove");
}

#[sqlx::test]
async fn deck_card_added_to_maybeboard(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, did) = deck_for(&app, "mayber").await;

    let counsel = card("Counterspell").mono("U").cmc(2.0).type_line("Instant");
    let sid = counsel.id();
    let oid = counsel.oracle_id().unwrap();
    seed_cards(&pool, &[counsel]).await;

    let (status, dc) = app
        .post(
            &format!("/api/deck/{did}/card"),
            json!({
                "scryfall_data_id": sid.to_string(),
                "oracle_id": oid.to_string(),
                "quantity": 1,
                "board": "maybeboard"
            }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "add to maybeboard: {dc}");
    assert_eq!(dc["board"], "maybeboard");

    let (_, full) = app.get(&format!("/api/deck/{did}"), Some(&token)).await;
    let entries = full["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["deck_card"]["board"], "maybeboard");
}

#[sqlx::test]
async fn clone_copies_the_cards(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, did) = deck_for(&app, "cloner").await;

    let a = card("Sol Ring").color_identity("").type_line("Artifact");
    let b = card("Arcane Signet").color_identity("").type_line("Artifact");
    let (a_sid, a_oid) = (a.id(), a.oracle_id().unwrap());
    let (b_sid, b_oid) = (b.id(), b.oracle_id().unwrap());
    seed_cards(&pool, &[a, b]).await;

    for (sid, oid, qty) in [(a_sid, a_oid, 1), (b_sid, b_oid, 3)] {
        let (status, _) = app
            .post(
                &format!("/api/deck/{did}/card"),
                json!({ "scryfall_data_id": sid.to_string(), "oracle_id": oid.to_string(), "quantity": qty }),
                Some(&token),
            )
            .await;
        assert_eq!(status, StatusCode::CREATED);
    }

    let (status, cloned) = app
        .post(&format!("/api/deck/{did}/clone"), json!({ "new_name": "Sol Ring Copy" }), Some(&token))
        .await;
    assert_eq!(status, StatusCode::CREATED, "clone: {cloned}");
    let clone_id = cloned["deck_id"].as_str().unwrap();

    // the clone carries both cards with their quantities
    let (_, full) = app.get(&format!("/api/deck/{clone_id}"), Some(&token)).await;
    let entries = full["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 2, "clone should copy both cards: {full}");
    let mut by_name: Vec<(String, i64)> = entries
        .iter()
        .map(|e| {
            (
                e["card"]["scryfall_data"]["name"].as_str().unwrap().to_string(),
                e["deck_card"]["quantity"].as_i64().unwrap(),
            )
        })
        .collect();
    by_name.sort();
    assert_eq!(by_name, vec![("Arcane Signet".to_string(), 3), ("Sol Ring".to_string(), 1)]);
}

#[sqlx::test]
async fn import_resolves_known_cards_and_reports_the_rest(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, did) = deck_for(&app, "importer").await;

    seed_cards(
        &pool,
        &[
            card("Lightning Bolt").mono("R").type_line("Instant"),
            card("Llanowar Elves").mono("G").type_line("Creature — Elf Druid"),
        ],
    )
    .await;

    // one resolvable, one bogus line
    let (status, result) = app
        .post(
            &format!("/api/deck/{did}/card/import"),
            json!({ "text": "2 Lightning Bolt\n1 Definitely Not A Real Card" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "import: {result}");
    let imported = result["imported"].as_array().unwrap();
    let unresolved = result["unresolved"].as_array().unwrap();
    assert_eq!(imported.len(), 1, "one card should resolve: {result}");
    assert_eq!(unresolved.len(), 1, "the bogus line should be unresolved: {result}");

    // the imported card really landed in the deck
    let (_, full) = app.get(&format!("/api/deck/{did}"), Some(&token)).await;
    let names: Vec<&str> = full["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["card"]["scryfall_data"]["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, vec!["Lightning Bolt"]);
}
