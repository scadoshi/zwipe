//! Commander maybeboard: per-user "maybe this commander" list. Adds are
//! idempotent and capped, removes are idempotent, and the GET hydrates
//! entries to full cards newest-save-first through `latest_cards`.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

mod common;

use axum::http::StatusCode;
use common::{TestApp, card, seed_cards};
use serde_json::json;

/// Names on the maybeboard, in returned (newest-first) order.
async fn maybeboard_names(app: &TestApp, token: &str) -> Vec<String> {
    let (status, body) = app.get("/api/user/commander-maybeboard", Some(token)).await;
    assert_eq!(status, StatusCode::OK, "maybeboard get: {body}");
    body.as_array()
        .unwrap()
        .iter()
        .map(|c| c["scryfall_data"]["name"].as_str().unwrap().to_string())
        .collect()
}

#[sqlx::test]
async fn add_list_remove_roundtrip(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, uid) = app.register("maybeboarder").await;
    app.verify_email(&uid).await;

    let first = card("First Pick").mono("R");
    let second = card("Second Pick").mono("G");
    let first_oracle = first.oracle_id().unwrap();
    let second_oracle = second.oracle_id().unwrap();
    seed_cards(&pool, &[first, second]).await;

    assert!(
        maybeboard_names(&app, &token).await.is_empty(),
        "fresh user starts empty"
    );

    for oracle in [first_oracle, second_oracle] {
        let (status, body) = app
            .post(
                &format!("/api/user/commander-maybeboard/{oracle}"),
                json!({}),
                Some(&token),
            )
            .await;
        assert_eq!(status, StatusCode::NO_CONTENT, "add: {body}");
    }

    // newest save first, hydrated to full cards
    let names = maybeboard_names(&app, &token).await;
    assert_eq!(names, vec!["Second Pick", "First Pick"], "newest first");

    // duplicate add is a no-op success and doesn't reorder or duplicate
    let (status, _) = app
        .post(
            &format!("/api/user/commander-maybeboard/{first_oracle}"),
            json!({}),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT, "duplicate add");
    let names = maybeboard_names(&app, &token).await;
    assert_eq!(names, vec!["Second Pick", "First Pick"], "unchanged");

    // remove one; removing it again is still a no-op success
    for _ in 0..2 {
        let (status, _) = app
            .delete(
                &format!("/api/user/commander-maybeboard/{second_oracle}"),
                Some(&token),
            )
            .await;
        assert_eq!(status, StatusCode::NO_CONTENT, "remove");
    }
    let names = maybeboard_names(&app, &token).await;
    assert_eq!(names, vec!["First Pick"], "one left after remove");
}

#[sqlx::test]
async fn unknown_and_invalid_oracle_ids_reject(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, uid) = app.register("rejector").await;
    app.verify_email(&uid).await;

    let (status, _) = app
        .post(
            &format!("/api/user/commander-maybeboard/{}", uuid::Uuid::new_v4()),
            json!({}),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "unknown oracle id");

    let (status, _) = app
        .post(
            "/api/user/commander-maybeboard/not-a-uuid",
            json!({}),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "malformed id");

    let (status, _) = app.get("/api/user/commander-maybeboard", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "no token");
}

#[sqlx::test]
async fn clear_wipes_the_whole_maybeboard(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, uid) = app.register("wiper").await;
    app.verify_email(&uid).await;

    let a = card("Alpha Cmdr").mono("R");
    let b = card("Bravo Cmdr").mono("G");
    let oracles = [a.oracle_id().unwrap(), b.oracle_id().unwrap()];
    seed_cards(&pool, &[a, b]).await;

    for oracle in oracles {
        let (status, _) = app
            .post(
                &format!("/api/user/commander-maybeboard/{oracle}"),
                json!({}),
                Some(&token),
            )
            .await;
        assert_eq!(status, StatusCode::NO_CONTENT);
    }
    assert_eq!(maybeboard_names(&app, &token).await.len(), 2);

    // clear empties it; clearing again is still a no-op success
    for _ in 0..2 {
        let (status, _) = app
            .delete("/api/user/commander-maybeboard", Some(&token))
            .await;
        assert_eq!(status, StatusCode::NO_CONTENT, "clear");
    }
    assert!(
        maybeboard_names(&app, &token).await.is_empty(),
        "cleared to empty"
    );
}

#[sqlx::test]
async fn cap_rejects_but_duplicates_still_ok(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, uid) = app.register("hoarder").await;
    app.verify_email(&uid).await;

    let fixtures: Vec<_> = (0..51)
        .map(|i| card(&format!("Cmdr {i:02}")).mono("R"))
        .collect();
    let oracles: Vec<_> = fixtures.iter().map(|f| f.oracle_id().unwrap()).collect();
    seed_cards(&pool, &fixtures).await;

    for oracle in &oracles[..50] {
        let (status, body) = app
            .post(
                &format!("/api/user/commander-maybeboard/{oracle}"),
                json!({}),
                Some(&token),
            )
            .await;
        assert_eq!(status, StatusCode::NO_CONTENT, "under cap: {body}");
    }

    // the 51st distinct commander is rejected...
    let (status, _) = app
        .post(
            &format!("/api/user/commander-maybeboard/{}", oracles[50]),
            json!({}),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "over cap");

    // ...but re-adding an existing entry at cap stays a no-op success
    let (status, _) = app
        .post(
            &format!("/api/user/commander-maybeboard/{}", oracles[0]),
            json!({}),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT, "duplicate at cap");

    assert_eq!(
        maybeboard_names(&app, &token).await.len(),
        50,
        "cap held at 50"
    );
}
