//! Deck-aware serve suppression: a skipped card must not come back through the
//! Add screen (`POST /api/deck/{id}/card/search`), unskipping restores it, and
//! clearing wipes the whole suppression set. This guards the `NOT EXISTS
//! (deck_card_suppressions ...)` clause in the deck-aware search.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

mod common;

use axum::http::StatusCode;
use common::{TestApp, card, seed_cards};
use serde_json::json;

/// Names returned by the deck-aware search (empty filter = the whole servable
/// pool for the deck).
async fn deck_search_names(app: &TestApp, deck_id: &str, token: &str) -> Vec<String> {
    let (status, body) = app
        .post(
            &format!("/api/deck/{deck_id}/card/search"),
            json!({}),
            Some(token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "deck search: {body}");
    body.as_array()
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
            "/api/deck",
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
            &format!("/api/deck/{did}/suppressions"),
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
            &format!("/api/deck/{did}/suppressions/{skip_oracle}"),
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
            "/api/deck",
            json!({ "name": "Clearable", "format": "commander" }),
            Some(&token),
        )
        .await;
    let did = deck["id"].as_str().unwrap().to_string();

    for oracle in [a_oracle, b_oracle] {
        let (status, _) = app
            .post(
                &format!("/api/deck/{did}/suppressions"),
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
        .delete(&format!("/api/deck/{did}/suppressions"), Some(&token))
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
