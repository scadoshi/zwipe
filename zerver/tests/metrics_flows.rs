//! Metrics ingest through the real router: the authed usage batch (which folds
//! `signals` into `commander_card_signal`) and the anonymous funnel event
//! (no auth, closed enum of kinds).
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::json;
use uuid::Uuid;

#[sqlx::test]
async fn usage_batch_folds_into_commander_card_signal(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, _) = app.register("swiper").await;

    let commander = Uuid::from_u128(0xC0);
    let card = Uuid::from_u128(0xCA);
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
                    "commander_oracle_id": commander.to_string(),
                    "card_oracle_id": card.to_string(),
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
    .bind(commander)
    .bind(card)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!((shown, added, skipped), (4, 3, 1), "signal delta persisted");
}

#[sqlx::test]
async fn usage_batch_signal_deltas_accumulate(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, _) = app.register("repeatswiper").await;

    let commander = Uuid::from_u128(0xB0);
    let card = Uuid::from_u128(0xBA);
    let batch = json!({
        "swipes_right": 1, "swipes_left": 0, "swipes_up": 0, "swipes_down": 0, "searches": 0,
        "signals": [{
            "commander_oracle_id": commander.to_string(),
            "card_oracle_id": card.to_string(),
            "shown": 2, "added": 1, "skipped": 1, "maybed": 0, "removed": 0
        }]
    });
    for _ in 0..2 {
        let (status, _) = app.post("/api/metrics/usage", batch.clone(), Some(&token)).await;
        assert_eq!(status, StatusCode::NO_CONTENT);
    }

    let (shown, added): (i64, i64) = sqlx::query_as(
        "SELECT shown, added FROM commander_card_signal \
         WHERE commander_oracle_id = $1 AND card_oracle_id = $2",
    )
    .bind(commander)
    .bind(card)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!((shown, added), (4, 2), "two flushes accumulate (upsert adds)");
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
        assert_eq!(status, StatusCode::NO_CONTENT, "kind {kind} accepted without auth");
    }

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM anonymous_events WHERE session_id = $1")
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
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "closed enum rejects unknown kinds");
}
