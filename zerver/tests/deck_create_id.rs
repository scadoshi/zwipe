//! Create deck and Clone with an id the client picks: a repeat of the same request, sent after
//! the client lost the first answer, answers the deck the first one made instead of a second
//! deck or a "name taken", and an id another user's deck holds is turned down.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![expect(
    clippy::unwrap_used,
    reason = "a test: helpers outside #[test] fns unwrap, and a panic is the failure report"
)]

mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::{Value, json};
use uuid::Uuid;
use zwipe_core::http::paths::{DECK_ROUTE, clone_deck_route, delete_deck_route};

async fn deck_count(pool: &sqlx::PgPool, user: &str) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM decks WHERE user_id = $1")
        .bind(user.parse::<Uuid>().unwrap())
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn create(app: &TestApp, token: &str, body: Value) -> (StatusCode, Value) {
    app.post(DECK_ROUTE, body, Some(token)).await
}

async fn verified(app: &TestApp, name: &str) -> (String, String) {
    let (token, uid) = app.register(name).await;
    app.verify_email(&uid).await;
    (token, uid)
}

#[sqlx::test]
async fn a_repeated_create_answers_the_deck_the_first_one_made(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, uid) = verified(&app, "repeater").await;
    let id = Uuid::from_u128(0x609_0001);
    let body = json!({ "name": "Krenko", "format": "commander", "id": id });

    let (status, first) = create(&app, &token, body.clone()).await;
    assert_eq!(status, StatusCode::CREATED, "{first}");
    assert_eq!(
        first["id"],
        id.to_string(),
        "the deck takes the client's id"
    );
    let (status, again) = create(&app, &token, body).await;
    assert_eq!(status, StatusCode::CREATED, "the repeat succeeds: {again}");
    assert_eq!(again["id"], first["id"]);
    assert_eq!(again["name"], first["name"]);
    assert_eq!(deck_count(&pool, &uid).await, 1, "one deck, not two");

    let (status, read) = app.get(&delete_deck_route(id), Some(&token)).await;
    assert_eq!(status, StatusCode::OK, "the recheck by id finds it: {read}");
}

#[sqlx::test]
async fn a_repeated_create_at_the_deck_cap_still_answers_its_deck(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    // An unverified account holds one deck, so the first create fills the cap.
    let (token, uid) = app.register("capped").await;
    let id = Uuid::from_u128(0x609_0002);
    let body = json!({ "name": "Only", "id": id });

    let (status, first) = create(&app, &token, body.clone()).await;
    assert_eq!(status, StatusCode::CREATED, "{first}");
    let (status, again) = create(&app, &token, body).await;
    assert_eq!(status, StatusCode::CREATED, "not the deck limit: {again}");
    assert_eq!(again["id"], id.to_string());
    assert_eq!(deck_count(&pool, &uid).await, 1);

    let (status, _) = create(&app, &token, json!({ "name": "Second" })).await;
    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "a new deck still meets the cap"
    );
}

#[sqlx::test]
async fn an_id_another_users_deck_holds_is_turned_down(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (owner, _) = verified(&app, "owner").await;
    let (other, other_id) = verified(&app, "other").await;
    let id = Uuid::from_u128(0x609_0003);
    let (status, _) = create(&app, &owner, json!({ "name": "Mine", "id": id })).await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, body) = create(&app, &other, json!({ "name": "Theirs", "id": id })).await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    let (status, source) = create(&app, &other, json!({ "name": "Source" })).await;
    assert_eq!(status, StatusCode::CREATED, "{source}");
    let source_id: Uuid = source["id"].as_str().unwrap().parse().unwrap();
    let (status, body) = app
        .post(
            &clone_deck_route(source_id),
            json!({ "new_name": "Copy", "id": id }),
            Some(&other),
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(deck_count(&pool, &other_id).await, 1, "only the source");
}

#[sqlx::test]
async fn a_repeated_clone_answers_the_copy_the_first_one_made(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, uid) = verified(&app, "cloner").await;
    let (status, source) = create(&app, &token, json!({ "name": "Source" })).await;
    assert_eq!(status, StatusCode::CREATED, "{source}");
    let source_id: Uuid = source["id"].as_str().unwrap().parse().unwrap();
    let id = Uuid::from_u128(0x609_0004);
    let body = json!({ "new_name": "Source copy", "id": id });

    let (status, first) = app
        .post(&clone_deck_route(source_id), body.clone(), Some(&token))
        .await;
    assert_eq!(status, StatusCode::CREATED, "{first}");
    assert_eq!(
        first["deck_id"],
        id.to_string(),
        "the copy takes the client's id"
    );
    let (status, again) = app
        .post(&clone_deck_route(source_id), body, Some(&token))
        .await;
    assert_eq!(status, StatusCode::CREATED, "the repeat succeeds: {again}");
    assert_eq!(again["deck_id"], first["deck_id"]);
    assert_eq!(deck_count(&pool, &uid).await, 2, "the source and one copy");
}

#[sqlx::test]
async fn a_create_or_clone_without_an_id_works_as_before(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, uid) = verified(&app, "shipped").await;
    let (status, deck) = create(&app, &token, json!({ "name": "Old app" })).await;
    assert_eq!(status, StatusCode::CREATED, "{deck}");
    let source_id: Uuid = deck["id"].as_str().unwrap().parse().unwrap();
    let (status, copy) = app
        .post(
            &clone_deck_route(source_id),
            json!({ "new_name": "Old app copy" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{copy}");
    assert_ne!(copy["deck_id"], deck["id"]);
    let (status, _) = create(&app, &token, json!({ "name": "Old app" })).await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "without an id a repeat is still a taken name"
    );
    assert_eq!(deck_count(&pool, &uid).await, 2);
}

#[sqlx::test]
async fn two_repeats_in_flight_at_once_make_one_deck(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, uid) = verified(&app, "racer").await;
    let id = Uuid::from_u128(0x609_0005);
    let body = json!({ "name": "Racing", "id": id });

    let (first, second) = tokio::join!(
        create(&app, &token, body.clone()),
        create(&app, &token, body)
    );
    for (status, deck) in [first, second] {
        assert_eq!(status, StatusCode::CREATED, "{deck}");
        assert_eq!(deck["id"], id.to_string());
    }
    assert_eq!(deck_count(&pool, &uid).await, 1);
}
