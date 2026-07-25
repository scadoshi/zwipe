//! Metrics ingest through the real router: the authed usage batch (which folds
//! `signals` into `commander_card_signal`) and the anonymous funnel event
//! (no auth, closed enum of kinds).
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

mod common;

use axum::http::StatusCode;
use common::{TestApp, card, seed_cards};
use serde_json::json;
use uuid::Uuid;

/// Creates a Commander deck with the given commander card set, returning the
/// deck id. Signals key on `deck_id` only; the server derives the commander.
async fn commander_deck(
    app: &TestApp,
    pool: &sqlx::PgPool,
    token: &str,
    commander_id: Uuid,
) -> Uuid {
    let (status, deck) = app
        .post(
            "/api/deck",
            json!({ "name": "Signal Deck", "format": "commander" }),
            Some(token),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "create: {deck}");
    let deck_id = Uuid::parse_str(deck["id"].as_str().unwrap()).unwrap();
    sqlx::query("UPDATE decks SET commander_id = $1 WHERE id = $2")
        .bind(commander_id)
        .bind(deck_id)
        .execute(pool)
        .await
        .unwrap();
    deck_id
}

#[sqlx::test]
async fn usage_batch_folds_into_commander_card_signal(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, _) = app.register("swiper").await;

    let atraxa = card("Atraxa, Praetors' Voice");
    let atraxa_id = atraxa.id();
    let atraxa_oracle = atraxa.oracle_id().unwrap();
    let bolt = card("Lightning Bolt");
    let bolt_oracle = bolt.oracle_id().unwrap();
    seed_cards(&pool, &[atraxa, bolt]).await;
    let deck_id = commander_deck(&app, &pool, &token, atraxa_id).await;

    let (status, _) = app
        .post(
            "/api/metrics/usage",
            json!({
                "swipes_right": 3,
                "swipes_left": 1,
                "swipes_up": 0,
                "swipes_down": 0,
                "searches": 2,
                "signals": [{
                    "card_oracle_id": bolt_oracle.to_string(),
                    "deck_id": deck_id.to_string(),
                    "shown": 4, "added": 3, "skipped": 1, "maybed": 0, "removed": 0
                }]
            }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT, "usage batch accepted");

    let (shown, added, skipped): (i64, i64, i64) = sqlx::query_as(
        "SELECT shown, added, skipped FROM commander_card_signal \
         WHERE commander_oracle_id = $1 AND card_oracle_id = $2",
    )
    .bind(atraxa_oracle)
    .bind(bolt_oracle)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!((shown, added, skipped), (4, 3, 1), "signal delta persisted");
}

#[sqlx::test]
async fn usage_batch_signal_deltas_accumulate(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, _) = app.register("repeatswiper").await;

    let atraxa = card("Atraxa, Praetors' Voice");
    let atraxa_id = atraxa.id();
    let atraxa_oracle = atraxa.oracle_id().unwrap();
    let bolt = card("Lightning Bolt");
    let bolt_oracle = bolt.oracle_id().unwrap();
    seed_cards(&pool, &[atraxa, bolt]).await;
    let deck_id = commander_deck(&app, &pool, &token, atraxa_id).await;

    let batch = json!({
        "swipes_right": 1, "swipes_left": 0, "swipes_up": 0, "swipes_down": 0, "searches": 0,
        "signals": [{
            "card_oracle_id": bolt_oracle.to_string(),
            "deck_id": deck_id.to_string(),
            "shown": 2, "added": 1, "skipped": 1, "maybed": 0, "removed": 0
        }]
    });
    for _ in 0..2 {
        let (status, _) = app
            .post("/api/metrics/usage", batch.clone(), Some(&token))
            .await;
        assert_eq!(status, StatusCode::NO_CONTENT);
    }

    let (shown, added): (i64, i64) = sqlx::query_as(
        "SELECT shown, added FROM commander_card_signal \
         WHERE commander_oracle_id = $1 AND card_oracle_id = $2",
    )
    .bind(atraxa_oracle)
    .bind(bolt_oracle)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        (shown, added),
        (4, 2),
        "two flushes accumulate (upsert adds)"
    );
}

#[sqlx::test]
async fn legacy_commander_field_is_ignored(pool: sqlx::PgPool) {
    // Phase 5S step 3: the legacy client-sent `commander_oracle_id` no longer
    // exists on the wire type and the server fallback is gone. A straggler
    // payload carrying it (and no deck_id) must be accepted — serde ignores
    // unknown fields — but land no commander-keyed signal.
    let app = TestApp::new(pool.clone());
    let (token, _) = app.register("straggler").await;

    let commander = Uuid::from_u128(0xC0);
    let (status, _) = app
        .post(
            "/api/metrics/usage",
            json!({
                "swipes_right": 1, "swipes_left": 0, "swipes_up": 0, "swipes_down": 0, "searches": 0,
                "signals": [{
                    "commander_oracle_id": commander.to_string(),
                    "card_oracle_id": Uuid::from_u128(0xCA).to_string(),
                    "shown": 1, "added": 1, "skipped": 0, "maybed": 0, "removed": 0
                }]
            }),
            Some(&token),
        )
        .await;
    assert_eq!(
        status,
        StatusCode::NO_CONTENT,
        "legacy payload still accepted"
    );

    let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM commander_card_signal")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(rows, 0, "client-sent commander no longer lands signal");
}

#[sqlx::test]
async fn anonymous_events_accept_the_three_kinds_no_auth(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let session = Uuid::from_u128(0x5E551047);

    for kind in ["app_opened", "register_viewed", "register_submitted"] {
        let (status, _) = app
            .post(
                "/api/metrics/anonymous",
                json!({ "session_id": session.to_string(), "kind": kind }),
                None, // no auth required
            )
            .await;
        assert_eq!(
            status,
            StatusCode::NO_CONTENT,
            "kind {kind} accepted without auth"
        );
    }

    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM anonymous_events WHERE session_id = $1")
            .bind(session)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count, 3, "all three funnel events recorded");
}

#[sqlx::test]
async fn anonymous_event_garbage_kind_rejected(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (status, _) = app
        .post(
            "/api/metrics/anonymous",
            json!({ "session_id": Uuid::from_u128(1).to_string(), "kind": "not_a_real_kind" }),
            None,
        )
        .await;
    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "closed enum rejects unknown kinds"
    );
}
