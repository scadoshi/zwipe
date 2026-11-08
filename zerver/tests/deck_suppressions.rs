//! Deck-aware serve suppression: a skipped card must not come back through the
//! Add screen (`POST /api/deck/{id}/card/search`), unskipping restores it, and
//! clearing wipes the whole suppression set. This guards the `NOT EXISTS
//! (deck_card_suppressions ...)` clause in the deck-aware search.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    reason = "a test: helpers outside #[test] fns unwrap and index, and the paging math multiplies small numbers; a panic is the failure report"
)]

use zwipe_core::http::contracts::deck_card::UNDO_QUERY;
use zwipe_core::http::paths::{
    DECK_ROUTE, create_deck_card_route, delete_deck_card_route, search_deck_cards_route,
    skip_deck_card_route, unskip_deck_card_route,
};
mod common;

use axum::http::StatusCode;
use common::{TestApp, card, seed_cards};
use serde_json::json;

/// Names returned by the deck-aware search (empty filter = the whole servable
/// pool for the deck).
async fn deck_search_names(app: &TestApp, deck_id: &str, token: &str) -> Vec<String> {
    let (status, body) = app
        .post(
            &search_deck_cards_route(deck_id.parse().unwrap()),
            json!({}),
            Some(token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "deck search: {body}");
    assert!(
        body.get("synergy_applied")
            .is_some_and(serde_json::Value::is_boolean),
        "deck search must carry synergy_applied: {body}"
    );
    body.get("cards")
        .and_then(|c| c.as_array())
        .unwrap()
        .iter()
        .map(|c| c["scryfall_data"]["name"].as_str().unwrap().to_string())
        .collect()
}

#[sqlx::test]
async fn skipped_card_is_excluded_then_unskip_restores(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, uid) = app.register("skipper").await;
    app.verify_email(&uid).await;

    let skip_me = card("Skip Me").mono("R");
    let keep_me = card("Keep Me").mono("R");
    let skip_oracle = skip_me.oracle_id().unwrap();
    seed_cards(&pool, &[skip_me, keep_me]).await;

    let (status, deck) = app
        .post(
            DECK_ROUTE,
            json!({ "name": "Suppressor", "format": "commander" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    let did = deck["id"].as_str().unwrap().to_string();

    // both cards serve before any suppression
    let before = deck_search_names(&app, &did, &token).await;
    assert!(
        before.contains(&"Skip Me".to_string()),
        "before: {before:?}"
    );
    assert!(
        before.contains(&"Keep Me".to_string()),
        "before: {before:?}"
    );

    // skip the target
    let (status, _) = app
        .post(
            &skip_deck_card_route(did.parse().unwrap()),
            json!({ "oracle_id": skip_oracle.to_string() }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT, "skip");

    // it no longer serves; the other card is untouched
    let after = deck_search_names(&app, &did, &token).await;
    assert!(
        !after.contains(&"Skip Me".to_string()),
        "skipped card must not re-serve: {after:?}"
    );
    assert!(after.contains(&"Keep Me".to_string()), "after: {after:?}");

    // unskip restores it (Clear-skips is the escape hatch)
    let (status, _) = app
        .delete(
            &unskip_deck_card_route(did.parse().unwrap(), skip_oracle),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT, "unskip");
    let restored = deck_search_names(&app, &did, &token).await;
    assert!(
        restored.contains(&"Skip Me".to_string()),
        "unskipped card returns: {restored:?}"
    );
}

#[sqlx::test]
async fn clear_suppressions_restores_all(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, uid) = app.register("clearer").await;
    app.verify_email(&uid).await;

    let a = card("Alpha").mono("R");
    let b = card("Bravo").mono("R");
    let a_oracle = a.oracle_id().unwrap();
    let b_oracle = b.oracle_id().unwrap();
    seed_cards(&pool, &[a, b]).await;

    let (_, deck) = app
        .post(
            DECK_ROUTE,
            json!({ "name": "Clearable", "format": "commander" }),
            Some(&token),
        )
        .await;
    let did = deck["id"].as_str().unwrap().to_string();

    for oracle in [a_oracle, b_oracle] {
        let (status, _) = app
            .post(
                &skip_deck_card_route(did.parse().unwrap()),
                json!({ "oracle_id": oracle.to_string() }),
                Some(&token),
            )
            .await;
        assert_eq!(status, StatusCode::NO_CONTENT);
    }
    assert!(
        deck_search_names(&app, &did, &token).await.is_empty(),
        "both suppressed → empty pool"
    );

    // clear reports the count removed and restores the pool
    let (status, cleared) = app
        .delete(&skip_deck_card_route(did.parse().unwrap()), Some(&token))
        .await;
    assert_eq!(status, StatusCode::OK, "clear: {cleared}");
    assert_eq!(cleared["cleared"], 2, "two suppressions cleared");

    let restored = deck_search_names(&app, &did, &token).await;
    assert_eq!(
        restored.len(),
        2,
        "all cards serve again after clear: {restored:?}"
    );
}

/// The swipe pile drains its own pool: every card it serves is skipped (or
/// added) before the next page, so each page must come from the top of what
/// is left. Pages are requested the way the shipped client does, with the
/// offset growing by the page size; an offset applied to the shrinking pool
/// would jump past cards that were never served.
#[sqlx::test]
async fn swiping_through_the_pile_serves_every_card(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, uid) = app.register("drainer").await;
    app.verify_email(&uid).await;

    let names = [
        "Alpha", "Bravo", "Charlie", "Delta", "Echo", "Foxtrot", "Golf", "Hotel",
    ];
    let cards: Vec<_> = names.iter().map(|n| card(n).mono("R")).collect();
    seed_cards(&pool, &cards).await;

    let (_, deck) = app
        .post(
            DECK_ROUTE,
            json!({ "name": "Drainer", "format": "commander" }),
            Some(&token),
        )
        .await;
    let did = deck["id"].as_str().unwrap().to_string();

    let page_size = 2;
    let mut served = Vec::new();
    for page in 0..names.len() {
        let (status, body) = app
            .post(
                &search_deck_cards_route(did.parse().unwrap()),
                json!({ "limit": page_size, "offset": page * page_size }),
                Some(&token),
            )
            .await;
        assert_eq!(status, StatusCode::OK, "deck search: {body}");
        let page_cards = body["cards"].as_array().unwrap().clone();
        if page_cards.is_empty() {
            break;
        }
        for c in page_cards {
            served.push(c["scryfall_data"]["name"].as_str().unwrap().to_string());
            let (status, _) = app
                .post(
                    &skip_deck_card_route(did.parse().unwrap()),
                    json!({ "oracle_id": c["scryfall_data"]["oracle_id"] }),
                    Some(&token),
                )
                .await;
            assert_eq!(status, StatusCode::NO_CONTENT, "skip");
        }
    }

    served.sort();
    assert_eq!(
        served, names,
        "every card serves once before the pile runs dry"
    );
}

/// Creates a commander deck and returns its id.
async fn new_deck(app: &TestApp, token: &str, name: &str) -> String {
    let (status, deck) = app
        .post(
            DECK_ROUTE,
            json!({ "name": name, "format": "commander" }),
            Some(token),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    deck["id"].as_str().unwrap().to_string()
}

/// Writes a suppression `days_ago` days old, as if it had aged in place.
async fn suppress_aged(
    pool: &sqlx::PgPool,
    deck_id: &str,
    oracle_id: uuid::Uuid,
    source: &str,
    days_ago: i32,
) {
    sqlx::query(
        "INSERT INTO deck_card_suppressions (deck_id, oracle_id, source, suppressed_at)
         VALUES ($1, $2, $3, now() - make_interval(days => $4))",
    )
    .bind(deck_id.parse::<uuid::Uuid>().unwrap())
    .bind(oracle_id)
    .bind(source)
    .bind(days_ago)
    .execute(pool)
    .await
    .unwrap();
}

#[sqlx::test]
async fn skips_last_a_week_and_removals_a_month(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, uid) = app.register("ager").await;
    app.verify_email(&uid).await;

    let fresh_skip = card("Fresh Skip").mono("R");
    let old_skip = card("Old Skip").mono("R");
    let fresh_removal = card("Fresh Removal").mono("R");
    let old_removal = card("Old Removal").mono("R");
    let rows = [
        (fresh_skip.oracle_id().unwrap(), "skip", 6),
        (old_skip.oracle_id().unwrap(), "skip", 8),
        (fresh_removal.oracle_id().unwrap(), "removal", 29),
        (old_removal.oracle_id().unwrap(), "removal", 31),
    ];
    seed_cards(&pool, &[fresh_skip, old_skip, fresh_removal, old_removal]).await;
    let did = new_deck(&app, &token, "Ager").await;
    for (oracle, source, days) in rows {
        suppress_aged(&pool, &did, oracle, source, days).await;
    }

    let mut served = deck_search_names(&app, &did, &token).await;
    served.sort();
    assert_eq!(
        served,
        ["Old Removal", "Old Skip"],
        "a skip lasts 7 days and a removal 30; older ones serve again"
    );
}

#[sqlx::test]
async fn returning_cards_serve_after_unseen_ones(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, uid) = app.register("returner").await;
    app.verify_email(&uid).await;

    let alpha = card("Alpha").mono("R");
    let alpha_oracle = alpha.oracle_id().unwrap();
    seed_cards(
        &pool,
        &[alpha, card("Bravo").mono("R"), card("Charlie").mono("R")],
    )
    .await;
    let did = new_deck(&app, &token, "Returner").await;
    suppress_aged(&pool, &did, alpha_oracle, "skip", 8).await;

    // Unsorted (no commander, so no synergy data) and sorted by name, where
    // Alpha would otherwise lead: either way the returning card comes last.
    for body in [json!({}), json!({ "order_by": "Name" })] {
        let (status, page) = app
            .post(
                &search_deck_cards_route(did.parse().unwrap()),
                body.clone(),
                Some(&token),
            )
            .await;
        assert_eq!(status, StatusCode::OK, "deck search: {page}");
        let names: Vec<&str> = page["cards"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["scryfall_data"]["name"].as_str().unwrap())
            .collect();
        assert_eq!(names.len(), 3, "{body}: {names:?}");
        assert_eq!(names.last(), Some(&"Alpha"), "{body}: {names:?}");
    }
}

/// Surfer's chain on Add, through the same routes the client calls: add,
/// skip, send to the maybeboard, add, then four undos in reverse (an undo
/// delete, an undo delete, unskip, an undo delete). Every undo fully reverses
/// its swipe, so all four cards serve again.
#[sqlx::test]
async fn undoing_every_swipe_serves_every_card_again(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, uid) = app.register("undoer").await;
    app.verify_email(&uid).await;

    let added = card("Champion Of Dusk").mono("B");
    let skipped = card("Elenda The Dusk Rose").mono("B");
    let maybe = card("Sorin Imperious Bloodlord").mono("B");
    let added_last = card("Indulgent Aristocrat").mono("B");
    seed_cards(
        &pool,
        &[
            added.clone(),
            skipped.clone(),
            maybe.clone(),
            added_last.clone(),
        ],
    )
    .await;
    let did = new_deck(&app, &token, "Undo Deck").await;
    let deck_id: uuid::Uuid = did.parse().unwrap();

    let add = |c: &common::CardFixture, board: &'static str| {
        json!({
            "scryfall_data_id": c.id().to_string(),
            "oracle_id": c.oracle_id().unwrap().to_string(),
            "quantity": 1,
            "board": board,
        })
    };
    for (body, what) in [
        (add(&added, "deck"), "add"),
        (add(&maybe, "maybeboard"), "maybe"),
        (add(&added_last, "deck"), "add last"),
    ] {
        let (status, resp) = app
            .post(&create_deck_card_route(deck_id), body, Some(&token))
            .await;
        assert_eq!(status, StatusCode::CREATED, "{what}: {resp}");
    }
    let (status, _) = app
        .post(
            &skip_deck_card_route(deck_id),
            json!({ "oracle_id": skipped.oracle_id().unwrap().to_string() }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT, "skip");

    // The undos, newest first, as the client sends them.
    let undo_delete = |c: &common::CardFixture| {
        format!("{}?{UNDO_QUERY}", delete_deck_card_route(deck_id, c.id()))
    };
    for c in [&added_last, &maybe] {
        let (status, _) = app.delete(&undo_delete(c), Some(&token)).await;
        assert_eq!(status, StatusCode::NO_CONTENT, "undo delete");
    }
    let (status, _) = app
        .delete(
            &unskip_deck_card_route(deck_id, skipped.oracle_id().unwrap()),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT, "undo skip");
    let (status, _) = app.delete(&undo_delete(&added), Some(&token)).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "undo add");

    let served = deck_search_names(&app, &did, &token).await;
    for name in [
        "Champion Of Dusk",
        "Elenda The Dusk Rose",
        "Sorin Imperious Bloodlord",
        "Indulgent Aristocrat",
    ] {
        assert!(
            served.contains(&name.to_string()),
            "{name} hidden after its undo: {served:?}"
        );
    }
}

/// A delete without `?undo=true` is a deliberate removal and still
/// suppresses the card: no flag, `false`, a malformed value and an unknown key
/// all delete (204) and hide it, so an old client and a bad flag behave as
/// before.
#[sqlx::test]
async fn a_delete_without_the_undo_flag_still_suppresses(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, uid) = app.register("remover").await;
    app.verify_email(&uid).await;

    let cards = [
        card("Plain Removal").mono("G"),
        card("False Flag").mono("G"),
        card("Banana Flag").mono("G"),
        card("Unknown Key").mono("G"),
    ];
    seed_cards(&pool, &cards).await;
    let did = new_deck(&app, &token, "Removal Deck").await;
    let deck_id: uuid::Uuid = did.parse().unwrap();

    for (c, query) in cards
        .iter()
        .zip(["", "?undo=false", "?undo=banana", "?other=1"])
    {
        let (status, resp) = app
            .post(
                &create_deck_card_route(deck_id),
                json!({
                    "scryfall_data_id": c.id().to_string(),
                    "oracle_id": c.oracle_id().unwrap().to_string(),
                    "quantity": 1,
                }),
                Some(&token),
            )
            .await;
        assert_eq!(status, StatusCode::CREATED, "{resp}");
        let path = format!("{}{query}", delete_deck_card_route(deck_id, c.id()));
        let (status, resp) = app.delete(&path, Some(&token)).await;
        assert_eq!(status, StatusCode::NO_CONTENT, "{path}: {resp}");
    }

    let served = deck_search_names(&app, &did, &token).await;
    for name in ["Plain Removal", "False Flag", "Banana Flag", "Unknown Key"] {
        assert!(
            !served.contains(&name.to_string()),
            "{name} served after a removal: {served:?}"
        );
    }
}
