//! Import honors a decklist's printing: `1 Card (SET) 123` lands that printing,
//! a line with no set lands the cheapest normal printing, a set the card lacks
//! falls back the same way, and no line ever resolves to an Art Series, token
//! or memorabilia printing.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "a test: helpers outside #[test] fns unwrap and index, and a panic is the failure report"
)]

mod common;

use axum::http::StatusCode;
use common::{TestApp, card, seed_cards};
use serde_json::json;
use uuid::Uuid;
use zwipe_core::http::paths::{DECK_ROUTE, import_deck_cards_route};

/// Creates a verified user and a commander deck, returns `(token, deck_id)`.
async fn deck_for(app: &TestApp, username: &str) -> (String, Uuid) {
    let (token, uid) = app.register(username).await;
    app.verify_email(&uid).await;
    let (status, deck) = app
        .post(
            DECK_ROUTE,
            json!({ "name": "Printing Deck", "format": "commander" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "deck create: {deck}");
    (token, deck["id"].as_str().unwrap().parse().unwrap())
}

/// Imports `text` in add mode and asserts it succeeded.
async fn import(app: &TestApp, token: &str, deck_id: Uuid, text: &str) {
    let (status, body) = app
        .post(
            &import_deck_cards_route(deck_id),
            json!({ "text": text }),
            Some(token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "import {text:?}: {body}");
}

/// Every deck row as `(oracle_id, scryfall_data_id)`.
async fn deck_rows(pool: &sqlx::PgPool, deck_id: Uuid) -> Vec<(Uuid, Uuid)> {
    sqlx::query_as("SELECT oracle_id, scryfall_data_id FROM deck_cards WHERE deck_id = $1")
        .bind(deck_id)
        .fetch_all(pool)
        .await
        .unwrap()
}

/// The printing `latest_cards` picks for a card.
async fn default_printing(pool: &sqlx::PgPool, oracle_id: Uuid) -> Uuid {
    sqlx::query_scalar("SELECT id FROM latest_cards WHERE oracle_id = $1")
        .bind(oracle_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

#[sqlx::test]
async fn a_named_printing_lands_that_printing(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, deck_id) = deck_for(&app, "printings").await;
    let oracle = Uuid::from_u128(0xde1);
    let isd = card("Delver of Secrets")
        .oracle(Some(oracle))
        .set("isd", "Innistrad")
        .collector_number("51");
    let inr = card("Delver of Secrets")
        .oracle(Some(oracle))
        .set("inr", "Innistrad Remastered")
        .collector_number("60");
    seed_cards(&pool, &[isd.clone(), inr.clone()]).await;

    import(&app, &token, deck_id, "1 Delver of Secrets (ISD) 51").await;
    assert_eq!(deck_rows(&pool, deck_id).await, vec![(oracle, isd.id())]);

    // A named printing moves the card the deck already holds.
    import(&app, &token, deck_id, "1 Delver of Secrets (INR) 60").await;
    assert_eq!(deck_rows(&pool, deck_id).await, vec![(oracle, inr.id())]);

    // The set alone, or a number the set lacks, takes that set's printing.
    import(&app, &token, deck_id, "1 Delver of Secrets (isd)").await;
    assert_eq!(deck_rows(&pool, deck_id).await, vec![(oracle, isd.id())]);
    import(&app, &token, deck_id, "1 Delver of Secrets (INR) 999").await;
    assert_eq!(deck_rows(&pool, deck_id).await, vec![(oracle, inr.id())]);

    // A bare name leaves the deck's printing alone.
    import(&app, &token, deck_id, "1 Delver of Secrets").await;
    assert_eq!(deck_rows(&pool, deck_id).await, vec![(oracle, inr.id())]);
}

#[sqlx::test]
async fn a_set_the_card_lacks_falls_back_to_the_default_printing(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, deck_id) = deck_for(&app, "fallback").await;
    let oracle = Uuid::from_u128(0xf00);
    seed_cards(
        &pool,
        &[
            card("Opt").oracle(Some(oracle)).set("xln", "Ixalan"),
            card("Opt").oracle(Some(oracle)).set("dom", "Dominaria"),
        ],
    )
    .await;

    import(&app, &token, deck_id, "4 Opt (M10) 51").await;
    let default = default_printing(&pool, oracle).await;
    assert_eq!(deck_rows(&pool, deck_id).await, vec![(oracle, default)]);
}

/// The Art Series card "Bloodline Keeper // Bloodline Keeper" has a front face
/// that matches the name `Bloodline Keeper`; the real card must win.
#[sqlx::test]
async fn no_line_resolves_to_an_art_series_token_or_memorabilia_printing(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, deck_id) = deck_for(&app, "artseries").await;
    let keeper = Uuid::from_u128(0xb1);
    let real = card("Bloodline Keeper // Lord of Lineage")
        .layout("transform")
        .oracle(Some(keeper))
        .set("isd", "Innistrad")
        .collector_number("90");
    // Sorts before the real card by name, so it would take the front-face key.
    let art_series = card("Bloodline Keeper // Bloodline Keeper")
        .layout("art_series")
        .set("isd", "Innistrad")
        .collector_number("90")
        .set_type("memorabilia");
    let gold_border = card("Bloodline Keeper // Lord of Lineage")
        .layout("transform")
        .oracle(Some(keeper))
        .set("wc11", "World Championship Decks 2011")
        .collector_number("90")
        .set_type("memorabilia");
    let token_card = card("Vampire").token().set("tisd", "Innistrad Tokens");
    seed_cards(&pool, &[real.clone(), art_series, gold_border, token_card]).await;

    import(&app, &token, deck_id, "1 Bloodline Keeper (ISD) 90").await;
    assert_eq!(deck_rows(&pool, deck_id).await, vec![(keeper, real.id())]);

    import(&app, &token, deck_id, "1 Bloodline Keeper (WC11) 90").await;
    assert_eq!(
        deck_rows(&pool, deck_id).await,
        vec![(keeper, real.id())],
        "a memorabilia printing is never named"
    );

    let (status, body) = app
        .post(
            &import_deck_cards_route(deck_id),
            json!({ "text": "1 Bloodline Keeper // Bloodline Keeper\n1 Vampire (TISD) 1" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let unresolved: Vec<&str> = body["unresolved"]
        .as_array()
        .unwrap()
        .iter()
        .map(|u| u["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        unresolved,
        vec!["Vampire"],
        "the token stays unresolved: {body}"
    );
    assert_eq!(
        deck_rows(&pool, deck_id).await,
        vec![(keeper, real.id())],
        "the Art Series name lands the playable card by its front face"
    );
}

#[sqlx::test]
async fn a_line_with_no_set_lands_the_cheapest_normal_printing(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, deck_id) = deck_for(&app, "cheapest").await;
    let bolt = Uuid::from_u128(0xb017);
    let cheap = card("Lightning Bolt")
        .oracle(Some(bolt))
        .set("m11", "Magic 2011")
        .usd("0.25");
    seed_cards(
        &pool,
        &[
            card("Lightning Bolt")
                .oracle(Some(bolt))
                .set("lea", "Limited Edition Alpha")
                .usd("28.99"),
            cheap.clone(),
            card("Lightning Bolt")
                .oracle(Some(bolt))
                .set("pmei", "Media Inserts")
                .promo(true)
                .usd("0.10"),
            card("Lightning Bolt")
                .oracle(Some(bolt))
                .set("2xm", "Double Masters")
                .finishes(&["foil"])
                .usd("0.05"),
            card("Lightning Bolt")
                .oracle(Some(bolt))
                .set("sld", "Secret Lair Drop")
                .lang("ja")
                .usd("0.01"),
        ],
    )
    .await;

    import(&app, &token, deck_id, "4 Lightning Bolt").await;
    assert_eq!(
        deck_rows(&pool, deck_id).await,
        vec![(bolt, cheap.id())],
        "not the promo, the foil-only or the Japanese printing"
    );

    // A set the card lacks falls back to the cheapest too.
    import(&app, &token, deck_id, "4 Lightning Bolt (M10) 1").await;
    assert_eq!(deck_rows(&pool, deck_id).await, vec![(bolt, cheap.id())]);
}

#[sqlx::test]
async fn a_card_with_no_priced_printing_lands_the_default_printing(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, deck_id) = deck_for(&app, "unpriced").await;
    let oracle = Uuid::from_u128(0x0b7);
    seed_cards(
        &pool,
        &[
            card("Opt").oracle(Some(oracle)).set("xln", "Ixalan"),
            card("Opt").oracle(Some(oracle)).set("dom", "Dominaria"),
        ],
    )
    .await;

    import(&app, &token, deck_id, "4 Opt").await;
    let default = default_printing(&pool, oracle).await;
    assert_eq!(deck_rows(&pool, deck_id).await, vec![(oracle, default)]);
}
