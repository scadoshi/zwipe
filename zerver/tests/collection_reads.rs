//! The collection's two reads against a real database: the summary a client checks before it
//! offers "In my collection", and the printings of one card the user owns. Both read only the
//! caller's own collection and need a session.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "a test: helpers outside #[test] fns unwrap and index, and a panic is the failure report"
)]

mod common;

use axum::http::StatusCode;
use common::{CardFixture, TestApp, card, seed_cards};
use serde_json::{Value, json};
use uuid::Uuid;
use zwipe_core::http::paths::{COLLECTION_SUMMARY_ROUTE, collection_printings_route};

/// Puts `quantity` copies of `printing` in `user`'s collection, in `finish`.
async fn own(pool: &sqlx::PgPool, user: &str, printing: &CardFixture, finish: &str, quantity: i32) {
    sqlx::query(
        "INSERT INTO collection_cards (user_id, scryfall_data_id, oracle_id, finish, quantity) \
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user.parse::<Uuid>().unwrap())
    .bind(printing.id())
    .bind(printing.oracle_id().unwrap())
    .bind(finish)
    .bind(quantity)
    .execute(pool)
    .await
    .unwrap();
}

struct World {
    sol_a: CardFixture,
    sol_b: CardFixture,
    sol_c: CardFixture,
    bolt: CardFixture,
}

async fn world(pool: &sqlx::PgPool) -> World {
    let sol = Uuid::from_u128(0x501);
    let world = World {
        sol_a: card("Sol Ring")
            .with_ids(Uuid::from_u128(0x5011), Some(sol))
            .set("c21", "Commander 2021"),
        sol_b: card("Sol Ring")
            .with_ids(Uuid::from_u128(0x5012), Some(sol))
            .set("c13", "Commander 2013"),
        sol_c: card("Sol Ring")
            .with_ids(Uuid::from_u128(0x5013), Some(sol))
            .set("cmm", "Commander Masters"),
        bolt: card("Lightning Bolt").mono("R"),
    };
    seed_cards(
        pool,
        &[
            world.sol_a.clone(),
            world.sol_b.clone(),
            world.sol_c.clone(),
            world.bolt.clone(),
        ],
    )
    .await;
    world
}

fn ids(body: &Value) -> Vec<String> {
    let mut ids: Vec<String> = body
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["scryfall_data"]["id"].as_str().unwrap().to_owned())
        .collect();
    ids.sort();
    ids
}

#[sqlx::test]
async fn the_summary_counts_the_callers_cards_and_printings(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let w = world(&pool).await;
    let (token, uid) = app.register("owner").await;

    let (status, body) = app.get(COLLECTION_SUMMARY_ROUTE, Some(&token)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        body,
        json!({ "cards": 0, "printings": 0 }),
        "an empty collection"
    );

    // Two Sol Ring printings, one in two finishes, and a Bolt: two cards, three printings.
    own(&pool, &uid, &w.sol_a, "nonfoil", 2).await;
    own(&pool, &uid, &w.sol_a, "foil", 1).await;
    own(&pool, &uid, &w.sol_b, "nonfoil", 1).await;
    own(&pool, &uid, &w.bolt, "nonfoil", 4).await;
    let (_, other) = app.register("other").await;
    own(&pool, &other, &w.sol_c, "nonfoil", 1).await;

    let (status, body) = app.get(COLLECTION_SUMMARY_ROUTE, Some(&token)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body, json!({ "cards": 2, "printings": 3 }));
}

#[sqlx::test]
async fn owned_printings_are_only_the_callers_printings_of_that_card(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let w = world(&pool).await;
    let (token, uid) = app.register("owner").await;
    own(&pool, &uid, &w.sol_a, "nonfoil", 1).await;
    own(&pool, &uid, &w.sol_a, "foil", 1).await;
    own(&pool, &uid, &w.sol_b, "nonfoil", 1).await;
    own(&pool, &uid, &w.bolt, "nonfoil", 1).await;
    let (_, other) = app.register("other").await;
    own(&pool, &other, &w.sol_c, "nonfoil", 1).await;

    let route = collection_printings_route(w.sol_a.oracle_id().unwrap());
    let (status, body) = app.get(&route, Some(&token)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let mut want = vec![w.sol_a.id().to_string(), w.sol_b.id().to_string()];
    want.sort();
    assert_eq!(
        ids(&body),
        want,
        "each owned printing once, never another user's"
    );

    let (status, body) = app
        .get(
            &collection_printings_route(Uuid::from_u128(0xdead)),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        body,
        json!([]),
        "a card the user doesn't own has no owned printings"
    );
}

#[sqlx::test]
async fn an_owned_non_english_printing_is_listed_as_search_counts_it(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let w = world(&pool).await;
    let sol_ja = card("Sol Ring")
        .with_ids(Uuid::from_u128(0x5014), w.sol_a.oracle_id())
        .set("c21", "Commander 2021")
        .lang("ja");
    seed_cards(&pool, std::slice::from_ref(&sol_ja)).await;
    let (token, uid) = app.register("owner").await;
    own(&pool, &uid, &sol_ja, "nonfoil", 1).await;

    let route = collection_printings_route(w.sol_a.oracle_id().unwrap());
    let (status, body) = app.get(&route, Some(&token)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(ids(&body), [sol_ja.id().to_string()]);
}

#[sqlx::test]
async fn the_reads_need_a_session_and_a_real_oracle_id(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let w = world(&pool).await;
    let route = collection_printings_route(w.bolt.oracle_id().unwrap());
    assert_eq!(
        app.get(COLLECTION_SUMMARY_ROUTE, None).await.0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(app.get(&route, None).await.0, StatusCode::UNAUTHORIZED);

    let (token, _) = app.register("owner").await;
    let (status, _) = app
        .get("/api/collection/printings/not-a-uuid", Some(&token))
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}
