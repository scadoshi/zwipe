//! A deck card's Flag through the real router: a patch sets and clears it, a board move and a
//! quantity patch keep it, removing the card drops it, a clone copies it as off, and a shared
//! deck shows the owner's flags.
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
use serde_json::{Value, json};
use uuid::Uuid;
use zwipe_core::http::paths::{
    DECK_ROUTE, clone_deck_route, create_deck_card_route, deck_route, delete_deck_card_route,
    share_deck_route, shared_deck_route, update_deck_card_route,
};

/// A verified user's commander deck holding one Llanowar Elves:
/// `(token, deck_id, card_id, oracle_id)`.
async fn deck_with_a_card(
    app: &TestApp,
    pool: &sqlx::PgPool,
    username: &str,
) -> (String, Uuid, Uuid, Uuid) {
    let (token, uid) = app.register(username).await;
    app.verify_email(&uid).await;
    let (status, deck) = app
        .post(
            DECK_ROUTE,
            json!({ "name": "Flags", "format": "commander" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "deck create: {deck}");
    let deck_id: Uuid = deck["id"].as_str().unwrap().parse().unwrap();
    let elf = card("Llanowar Elves").type_line("Creature");
    let (sid, oid) = (elf.id(), elf.oracle_id().unwrap());
    seed_cards(pool, &[elf]).await;
    let (status, added) = app
        .post(
            &create_deck_card_route(deck_id),
            json!({ "scryfall_data_id": sid.to_string(), "oracle_id": oid.to_string(), "quantity": 2 }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "add: {added}");
    assert_eq!(
        added["flagged"], false,
        "a new card starts unflagged: {added}"
    );
    (token, deck_id, sid, oid)
}

/// The deck card for `card_id` in a full deck reply.
fn entry(deck: &Value, card_id: Uuid) -> Value {
    deck["entries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["deck_card"]["scryfall_data_id"] == card_id.to_string())
        .unwrap()["deck_card"]
        .clone()
}

async fn flag(
    app: &TestApp,
    token: &str,
    deck_id: Uuid,
    card_id: Uuid,
    on: Value,
) -> (StatusCode, Value) {
    app.patch(
        &update_deck_card_route(deck_id, card_id),
        json!({ "flagged": on }),
        Some(token),
    )
    .await
}

#[sqlx::test]
async fn a_flag_patch_sets_and_clears_the_flag(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, deck_id, sid, _) = deck_with_a_card(&app, &pool, "flagger").await;

    let (status, body) = flag(&app, &token, deck_id, sid, json!(true)).await;
    assert_eq!(status, StatusCode::OK, "flag: {body}");
    assert_eq!(body["flagged"], true, "{body}");
    let (_, deck) = app.get(&deck_route(deck_id), Some(&token)).await;
    assert_eq!(
        entry(&deck, sid)["flagged"],
        true,
        "the reloaded deck keeps it: {deck}"
    );

    let (status, body) = flag(&app, &token, deck_id, sid, json!(false)).await;
    assert_eq!(status, StatusCode::OK, "unflag: {body}");
    assert_eq!(body["flagged"], false, "{body}");

    let (status, body) = flag(&app, &token, deck_id, sid, Value::Null).await;
    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "a null flag is refused: {body}"
    );
}

#[sqlx::test]
async fn a_board_move_and_a_quantity_patch_keep_the_flag(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, deck_id, sid, _) = deck_with_a_card(&app, &pool, "mover").await;
    let (status, body) = flag(&app, &token, deck_id, sid, json!(true)).await;
    assert_eq!(status, StatusCode::OK, "flag: {body}");

    let (status, moved) = app
        .patch(
            &update_deck_card_route(deck_id, sid),
            json!({ "board": "maybeboard" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "move: {moved}");
    assert_eq!(moved["board"], "maybeboard");
    assert_eq!(moved["flagged"], true, "the move keeps the flag: {moved}");

    let (status, fixed) = app
        .patch(
            &update_deck_card_route(deck_id, sid),
            json!({ "quantity": 1 }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "fix: {fixed}");
    assert_eq!(
        fixed["flagged"], true,
        "a patch without the field leaves it alone: {fixed}"
    );
}

#[sqlx::test]
async fn removing_a_card_drops_its_flag(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, deck_id, sid, oid) = deck_with_a_card(&app, &pool, "remover").await;
    let (status, body) = flag(&app, &token, deck_id, sid, json!(true)).await;
    assert_eq!(status, StatusCode::OK, "flag: {body}");

    let (status, body) = app
        .delete(&delete_deck_card_route(deck_id, sid), Some(&token))
        .await;
    assert!(status.is_success(), "remove: {body}");
    let (status, added) = app
        .post(
            &create_deck_card_route(deck_id),
            json!({ "scryfall_data_id": sid.to_string(), "oracle_id": oid.to_string(), "quantity": 1 }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "re-add: {added}");
    assert_eq!(
        added["flagged"], false,
        "the card comes back unflagged: {added}"
    );
}

#[sqlx::test]
async fn a_clone_copies_flags_as_off(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, deck_id, sid, _) = deck_with_a_card(&app, &pool, "cloner").await;
    let (status, body) = flag(&app, &token, deck_id, sid, json!(true)).await;
    assert_eq!(status, StatusCode::OK, "flag: {body}");

    let (status, cloned) = app
        .post(
            &clone_deck_route(deck_id),
            json!({ "new_name": "Flags Copy" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "clone: {cloned}");
    let clone_id: Uuid = cloned["deck_id"].as_str().unwrap().parse().unwrap();
    let (_, copy) = app.get(&deck_route(clone_id), Some(&token)).await;
    assert_eq!(
        entry(&copy, sid)["flagged"],
        false,
        "the copy starts unflagged: {copy}"
    );
    let (_, original) = app.get(&deck_route(deck_id), Some(&token)).await;
    assert_eq!(
        entry(&original, sid)["flagged"],
        true,
        "the original keeps its flag"
    );
}

#[sqlx::test]
async fn a_shared_deck_shows_the_owner_s_flags(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, deck_id, sid, _) = deck_with_a_card(&app, &pool, "sharer").await;
    let (status, body) = flag(&app, &token, deck_id, sid, json!(true)).await;
    assert_eq!(status, StatusCode::OK, "flag: {body}");

    let (status, share) = app
        .post(&share_deck_route(deck_id), json!({}), Some(&token))
        .await;
    assert!(status.is_success(), "share: {share}");
    let share_token: Uuid = share["share_token"].as_str().unwrap().parse().unwrap();
    let (status, shared) = app.get(&shared_deck_route(share_token), None).await;
    assert_eq!(status, StatusCode::OK, "shared: {shared}");
    assert_eq!(
        entry(&shared, sid)["flagged"],
        true,
        "a shared deck shows the owner's flag: {shared}"
    );
}
