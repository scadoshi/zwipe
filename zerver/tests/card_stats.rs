//! `GET /api/card/{oracle_id}/stats`: one card's public numbers, read live. Its mainboard decks
//! and the vested MVPs among them, its pooled swipes, the decks it leads and its Swipe select
//! picks, signed out and cached an hour.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![expect(
    clippy::unwrap_used,
    reason = "a test: helpers outside #[test] fns unwrap, and a panic is the failure report"
)]

use axum::http::{Method, StatusCode, header};
use common::{TestApp, card, seed_cards};
use serde_json::{Value, json};
use uuid::Uuid;
use zwipe_core::http::contracts::card::HttpCardStats;
use zwipe_core::http::paths::{CARD_ROUTE, DECK_ROUTE, card_stats_route};

mod common;

/// A fresh commander deck for the signed-in user.
async fn deck(app: &TestApp, token: &str, name: &str) -> Uuid {
    let (status, deck) = app
        .post(
            DECK_ROUTE,
            json!({ "name": name, "format": "commander" }),
            Some(token),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "create {name}: {deck}");
    Uuid::parse_str(deck.get("id").and_then(Value::as_str).unwrap()).unwrap()
}

/// Puts `printing` on `board` of `deck`, starred `mvp_days_ago` days ago when given.
async fn put(
    pool: &sqlx::PgPool,
    deck: Uuid,
    printing: (Uuid, Uuid),
    board: &str,
    mvp_days_ago: Option<i32>,
) {
    sqlx::query(
        "INSERT INTO deck_cards (deck_id, scryfall_data_id, oracle_id, quantity, board, mvp_at) \
         VALUES ($1, $2, $3, 1, $4, now() - make_interval(days => $5))",
    )
    .bind(deck)
    .bind(printing.0)
    .bind(printing.1)
    .bind(board)
    .bind(mvp_days_ago)
    .execute(pool)
    .await
    .unwrap();
}

/// Asks for `oracle_id`'s stats signed out, with the answer's headers.
async fn stats(app: &TestApp, oracle_id: Uuid) -> (StatusCode, header::HeaderMap, Value) {
    let (status, headers, bytes) = app
        .send_bytes(Method::GET, &card_stats_route(oracle_id))
        .await;
    let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, headers, body)
}

#[sqlx::test]
async fn a_card_s_stats_count_every_source_signed_out_and_cached_an_hour(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, user_id) = app.register("statsowner").await;
    app.verify_email(&user_id).await;
    let atraxa_oracle = Uuid::from_u128(0xa7);
    let atraxa = card("Atraxa, Praetors' Voice").oracle(Some(atraxa_oracle));
    let atraxa_reprint = card("Atraxa, Praetors' Voice")
        .oracle(Some(atraxa_oracle))
        .set("B", "Beta Set");
    let ring = card("Sol Ring");
    let ring_printing = (ring.id(), ring.oracle_id().unwrap());
    let ring_oracle = ring_printing.1;
    let (atraxa_id, reprint_id) = (atraxa.id(), atraxa_reprint.id());
    seed_cards(&pool, &[atraxa, atraxa_reprint, ring]).await;

    let fresh = deck(&app, &token, "Fresh star").await;
    let vested = deck(&app, &token, "Vested star").await;
    let maybe = deck(&app, &token, "Maybe only").await;
    put(&pool, fresh, ring_printing, "deck", Some(1)).await;
    put(&pool, vested, ring_printing, "deck", Some(4)).await;
    put(&pool, maybe, ring_printing, "maybeboard", None).await;
    for (sql, printing, deck) in [
        (
            "UPDATE decks SET commander_id = $1 WHERE id = $2",
            atraxa_id,
            fresh,
        ),
        (
            "UPDATE decks SET partner_commander_id = $1 WHERE id = $2",
            reprint_id,
            vested,
        ),
    ] {
        sqlx::query(sql)
            .bind(printing)
            .bind(deck)
            .execute(&pool)
            .await
            .unwrap();
    }
    sqlx::query(
        "INSERT INTO card_swipe_signal (card_oracle_id, shown, added, skipped, maybed, removed) \
         VALUES ($1, 50, 30, 15, 5, 3)",
    )
    .bind(ring_oracle)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO commander_select_signal (commander_oracle_id, shown, selected, skipped) \
         VALUES ($1, 40, 10, 30)",
    )
    .bind(atraxa_oracle)
    .execute(&pool)
    .await
    .unwrap();

    let (status, headers, body) = stats(&app, ring_oracle).await;
    assert_eq!(status, StatusCode::OK, "ring stats: {body}");
    assert_eq!(
        headers
            .get(header::CACHE_CONTROL)
            .and_then(|v| v.to_str().ok()),
        Some("public, max-age=3600"),
        "the hourly edge cache"
    );
    assert_eq!(
        body,
        json!({
            "in_decks": 2,
            "mvp_decks": 1,
            "swipes": { "shown": 50, "added": 30, "skipped": 15, "maybed": 5, "removed": 3 },
            "decks_led": 0,
            "commander_picks": null,
        }),
        "a maybeboard copy is in no deck and a star under 3 days old isn't counted"
    );

    let (status, _, body) = stats(&app, atraxa_oracle).await;
    assert_eq!(status, StatusCode::OK, "atraxa stats: {body}");
    let atraxa: HttpCardStats = serde_json::from_value(body).unwrap();
    assert_eq!(
        atraxa.decks_led,
        Some(2),
        "a deck it leads by either printing, as commander or partner"
    );
    assert_eq!(
        atraxa
            .commander_picks
            .map(|picks| (picks.shown, picks.selected)),
        Some((40, 10))
    );
    assert_eq!(atraxa.swipes, None, "nobody has swiped it on an Add stack");
}

#[sqlx::test]
async fn a_card_nobody_has_touched_answers_zeros_not_404(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let quiet = card("Quiet Card");
    let quiet_oracle = quiet.oracle_id().unwrap();
    seed_cards(&pool, &[quiet]).await;

    for oracle_id in [quiet_oracle, Uuid::from_u128(0xdead)] {
        let (status, _, body) = stats(&app, oracle_id).await;
        assert_eq!(status, StatusCode::OK, "{oracle_id}: {body}");
        assert_eq!(
            body,
            json!({
                "in_decks": 0,
                "mvp_decks": 0,
                "swipes": null,
                "decks_led": 0,
                "commander_picks": null,
            }),
            "{oracle_id}"
        );
    }
}

#[sqlx::test]
async fn a_malformed_oracle_id_is_refused(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (status, _, _) = app
        .send_bytes(Method::GET, &format!("{CARD_ROUTE}/not-a-uuid/stats"))
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}
