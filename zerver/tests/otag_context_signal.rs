//! Phase 5 (otags): the generalized-context per-otag signal — the cross-format
//! moat dataset, shipped dark. An add-stack usage batch must credit
//! `otag_context_signal`, one row per OTAG OF THE SWIPED CARD, keyed by the
//! deck's generalized context: the commander (every existing client) or, for a
//! non-Commander deck, its `(format, color identity)` derived server-side from
//! `deck_id`. Nothing about otags or format/CI is on the wire.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

mod common;

use axum::http::StatusCode;
use common::{TestApp, card, seed_cards};
use serde_json::json;
use uuid::Uuid;

#[sqlx::test]
async fn commander_context_credits_each_swiped_card_otag(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, _) = app.register("edh").await;

    // A card carrying two otags; every existing client already sends the
    // commander, so this path works with no client change.
    let bolt = card("Lightning Bolt").oracle_tags(&["burn", "spot-removal"]);
    let bolt_oracle = bolt.oracle_id().unwrap();
    seed_cards(&pool, &[bolt]).await;

    let commander = Uuid::from_u128(0xC0);
    let (status, _) = app
        .post(
            "/api/metrics/usage",
            json!({
                "swipes_right": 2, "swipes_left": 1, "swipes_up": 0, "swipes_down": 0, "searches": 0,
                "signals": [{
                    "commander_oracle_id": commander.to_string(),
                    "card_oracle_id": bolt_oracle.to_string(),
                    "shown": 3, "added": 2, "skipped": 1, "maybed": 0, "removed": 0
                }]
            }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let context = format!("commander:{commander}");
    let rows: Vec<(String, i64, i64, i64)> = sqlx::query_as(
        "SELECT oracle_tag, shown, added, skipped FROM otag_context_signal \
         WHERE context_key = $1 ORDER BY oracle_tag",
    )
    .bind(&context)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        rows,
        vec![
            ("burn".to_string(), 3, 2, 1),
            ("spot-removal".to_string(), 3, 2, 1),
        ],
        "each of the swiped card's otags is credited under the commander context",
    );
}

#[sqlx::test]
async fn non_commander_deck_credits_format_and_color_identity(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, _) = app.register("modernist").await;

    let blaze = card("Fireblast").oracle_tags(&["burn"]).color_identity("R");
    let blaze_id = blaze.id();
    let blaze_oracle = blaze.oracle_id().unwrap();
    seed_cards(&pool, &[blaze]).await;

    // A non-Commander deck: format modern, one red mainboard card → CI "R".
    let (status, deck) = app
        .post(
            "/api/deck",
            json!({ "name": "Burn", "format": "modern" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "create: {deck}");
    let deck_id = Uuid::parse_str(deck["id"].as_str().unwrap()).unwrap();

    sqlx::query(
        "INSERT INTO deck_cards (deck_id, scryfall_data_id, oracle_id, quantity, board) \
         VALUES ($1, $2, $3, 1, 'deck')",
    )
    .bind(deck_id)
    .bind(blaze_id)
    .bind(blaze_oracle)
    .execute(&pool)
    .await
    .unwrap();

    // No commander; deck_id carries the context. A future non-EDH client sends
    // this shape — older clients never do, so this branch is dark until they ship.
    let (status, _) = app
        .post(
            "/api/metrics/usage",
            json!({
                "swipes_right": 1, "swipes_left": 0, "swipes_up": 0, "swipes_down": 0, "searches": 0,
                "signals": [{
                    "commander_oracle_id": Uuid::nil().to_string(),
                    "card_oracle_id": blaze_oracle.to_string(),
                    "deck_id": deck_id.to_string(),
                    "shown": 1, "added": 1, "skipped": 0, "maybed": 0, "removed": 0
                }]
            }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (added, shown): (i64, i64) = sqlx::query_as(
        "SELECT added, shown FROM otag_context_signal \
         WHERE context_key = 'format_ci:modern:R' AND oracle_tag = 'burn'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        (added, shown),
        (1, 1),
        "credited under the (format, CI) context derived from the deck",
    );
}

#[sqlx::test]
async fn deck_id_ownership_is_enforced(pool: sqlx::PgPool) {
    // A client can't attribute signal to a deck it doesn't own: the (format, CI)
    // lookup is scoped to the caller's decks, so a foreign deck_id yields no
    // context and no otag credit.
    let app = TestApp::new(pool.clone());
    let (owner_token, _) = app.register("owner").await;
    let (attacker_token, _) = app.register("attacker").await;

    let blaze = card("Fireblast").oracle_tags(&["burn"]).color_identity("R");
    let blaze_oracle = blaze.oracle_id().unwrap();
    seed_cards(&pool, &[blaze]).await;

    let (status, deck) = app
        .post(
            "/api/deck",
            json!({ "name": "Burn", "format": "modern" }),
            Some(&owner_token),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "create: {deck}");
    let deck_id = Uuid::parse_str(deck["id"].as_str().unwrap()).unwrap();

    let (status, _) = app
        .post(
            "/api/metrics/usage",
            json!({
                "swipes_right": 1, "swipes_left": 0, "swipes_up": 0, "swipes_down": 0, "searches": 0,
                "signals": [{
                    "commander_oracle_id": Uuid::nil().to_string(),
                    "card_oracle_id": blaze_oracle.to_string(),
                    "deck_id": deck_id.to_string(),
                    "shown": 1, "added": 1, "skipped": 0, "maybed": 0, "removed": 0
                }]
            }),
            Some(&attacker_token),
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM otag_context_signal")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        count, 0,
        "signal for a deck the caller doesn't own is dropped"
    );
}

#[sqlx::test]
async fn signal_without_commander_or_deck_credits_no_otag_rows(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, _) = app.register("ctxless").await;

    let shock = card("Shock").oracle_tags(&["burn"]);
    let shock_oracle = shock.oracle_id().unwrap();
    seed_cards(&pool, &[shock]).await;

    let (status, _) = app
        .post(
            "/api/metrics/usage",
            json!({
                "swipes_right": 1, "swipes_left": 0, "swipes_up": 0, "swipes_down": 0, "searches": 0,
                "signals": [{
                    "commander_oracle_id": Uuid::nil().to_string(),
                    "card_oracle_id": shock_oracle.to_string(),
                    "shown": 1, "added": 1, "skipped": 0, "maybed": 0, "removed": 0
                }]
            }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM otag_context_signal")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        count, 0,
        "no context (no commander, no deck_id) → no otag credit",
    );
}
