//! Deck lifecycle through the real router: profile CRUD, the unverified-email
//! deck cap, duplicate-name rejection, cross-user isolation (IDOR), and clone.
//!
//! Card-level tests (add/remove, card-copy on clone, suppressions) need real
//! `cards` rows and live in the card-serving slice with the fixture builder.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::json;

#[sqlx::test]
async fn deck_profile_lifecycle(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (token, _uid) = app.register("alice").await;

    // create
    let (status, deck) = app
        .post(
            "/api/deck",
            json!({ "name": "My First Deck", "format": "commander" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "create: {deck}");
    let id = deck["id"].as_str().unwrap().to_string();
    assert_eq!(deck["name"], "My First Deck");

    // get profile + full deck
    let (status, prof) = app.get(&format!("/api/deck/profile/{id}"), Some(&token)).await;
    assert_eq!(status, StatusCode::OK, "get profile: {prof}");
    assert_eq!(prof["name"], "My First Deck");
    let (status, _full) = app.get(&format!("/api/deck/{id}"), Some(&token)).await;
    assert_eq!(status, StatusCode::OK);

    // list contains it
    let (status, list) = app.get("/api/deck", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        list.as_array().unwrap().iter().any(|d| d["id"] == deck["id"]),
        "deck list missing the created deck: {list}"
    );

    // update the name. The Opdate fields without #[serde(default)] must be sent;
    // "Unchanged" is the wire form of Opdate::Unchanged.
    let (status, updated) = app
        .put(
            &format!("/api/deck/{id}"),
            json!({
                "name": "Renamed",
                "commander_id": "Unchanged",
                "partner_commander_id": "Unchanged",
                "background_id": "Unchanged",
                "signature_spell_id": "Unchanged",
                "format": "Unchanged"
            }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "update: {updated}");
    let (_, prof) = app.get(&format!("/api/deck/profile/{id}"), Some(&token)).await;
    assert_eq!(prof["name"], "Renamed");

    // delete => 204, then gone
    let (status, _) = app.delete(&format!("/api/deck/{id}"), Some(&token)).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "delete");
    let (status, _) = app.get(&format!("/api/deck/profile/{id}"), Some(&token)).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "profile after delete");
}

#[sqlx::test]
async fn unverified_deck_cap_then_verify_unlocks(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (token, uid) = app.register("bob").await;

    // unverified cap = 1 deck
    let (s1, _) = app.post("/api/deck", json!({ "name": "Deck 1" }), Some(&token)).await;
    assert_eq!(s1, StatusCode::CREATED);
    let (s2, e) = app.post("/api/deck", json!({ "name": "Deck 2" }), Some(&token)).await;
    assert_eq!(s2, StatusCode::UNPROCESSABLE_ENTITY, "unverified 2nd deck should be capped: {e}");

    // verifying lifts the cap
    app.verify_email(&uid).await;
    let (s3, _) = app.post("/api/deck", json!({ "name": "Deck 2" }), Some(&token)).await;
    assert_eq!(s3, StatusCode::CREATED, "verified 2nd deck should succeed");
}

#[sqlx::test]
async fn duplicate_deck_name_rejected(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (token, uid) = app.register("carol").await;
    app.verify_email(&uid).await; // lift the cap so the 2nd create reaches the dup check

    let (s1, _) = app.post("/api/deck", json!({ "name": "Twin" }), Some(&token)).await;
    assert_eq!(s1, StatusCode::CREATED);
    let (s2, _) = app.post("/api/deck", json!({ "name": "Twin" }), Some(&token)).await;
    assert_eq!(s2, StatusCode::UNPROCESSABLE_ENTITY, "duplicate name should be rejected");
}

#[sqlx::test]
async fn cannot_touch_another_users_deck(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (a_token, _) = app.register("dave").await;
    let (_, deck) = app.post("/api/deck", json!({ "name": "Private" }), Some(&a_token)).await;
    let id = deck["id"].as_str().unwrap().to_string();

    // 404 (not 403) — another user's deck must look nonexistent, no existence leak.
    let (b_token, _) = app.register("erin").await;
    let (status, _) = app.get(&format!("/api/deck/{id}"), Some(&b_token)).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "B reading A's deck must 404");
    let (status, _) = app.delete(&format!("/api/deck/{id}"), Some(&b_token)).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "B deleting A's deck must 404");
    // A's deck still there
    let (status, _) = app.get(&format!("/api/deck/{id}"), Some(&a_token)).await;
    assert_eq!(status, StatusCode::OK, "A's deck should be untouched");
}

#[sqlx::test]
async fn clone_creates_a_new_deck(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (token, uid) = app.register("frank").await;
    app.verify_email(&uid).await;

    let (_, deck) = app
        .post("/api/deck", json!({ "name": "Original", "format": "commander" }), Some(&token))
        .await;
    let id = deck["id"].as_str().unwrap().to_string();

    let (status, cloned) = app
        .post(&format!("/api/deck/{id}/clone"), json!({ "new_name": "Original Copy" }), Some(&token))
        .await;
    assert_eq!(status, StatusCode::CREATED, "clone: {cloned}");
    let clone_id = cloned["deck_id"].as_str().unwrap();
    assert_ne!(clone_id, id, "clone must get a new id");

    // now two decks exist, and the clone carries the new name
    let (_, list) = app.get("/api/deck", Some(&token)).await;
    assert_eq!(list.as_array().unwrap().len(), 2);
    let (_, clone_prof) = app.get(&format!("/api/deck/profile/{clone_id}"), Some(&token)).await;
    assert_eq!(clone_prof["name"], "Original Copy");
}
