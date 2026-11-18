//! "In my collection" (`owned_only`) on card search against a real database: a card counts as
//! owned when the searching user has any printing of it, only their own collection counts, and
//! a search without the flag is unchanged.
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
use zwipe_core::http::paths::{
    DECK_ROUTE, SEARCH_CARDS_ROUTE, SEARCH_COMMANDERS_ROUTE, search_deck_cards_route,
};

/// The names in a search reply, sorted: a bare array, or a deck search's `{ cards }`.
fn names(body: &Value) -> Vec<String> {
    let cards = body.get("cards").unwrap_or(body);
    let mut names: Vec<String> = cards
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["scryfall_data"]["name"].as_str().unwrap().to_owned())
        .collect();
    names.sort();
    names
}

/// Puts one copy of `printing` in `user`'s collection.
async fn own(pool: &sqlx::PgPool, user: &str, printing: &CardFixture) {
    sqlx::query(
        "INSERT INTO collection_cards (user_id, scryfall_data_id, oracle_id, finish, quantity) \
         VALUES ($1, $2, $3, 'nonfoil', 1)",
    )
    .bind(user.parse::<Uuid>().unwrap())
    .bind(printing.id())
    .bind(printing.oracle_id().unwrap())
    .execute(pool)
    .await
    .unwrap();
}

struct World {
    /// Sol Ring's two printings; search serves one of them.
    sol_a: CardFixture,
    sol_b: CardFixture,
    bolt: CardFixture,
    elves: CardFixture,
    krenko: CardFixture,
    zada: CardFixture,
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
        bolt: card("Lightning Bolt").mono("R"),
        elves: card("Llanowar Elves").mono("G"),
        krenko: card("Krenko, Mob Boss")
            .mono("R")
            .type_line("Legendary Creature — Goblin Warrior"),
        zada: card("Zada, Hedron Grinder")
            .mono("R")
            .type_line("Legendary Creature — Goblin Ally"),
    };
    seed_cards(
        pool,
        &[
            world.sol_a.clone(),
            world.sol_b.clone(),
            world.bolt.clone(),
            world.elves.clone(),
            world.krenko.clone(),
            world.zada.clone(),
        ],
    )
    .await;
    world
}

#[sqlx::test]
async fn owned_only_keeps_cards_with_any_printing_in_the_users_collection(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let w = world(&pool).await;
    let (token, uid) = app.register("owner").await;
    // Own the Sol Ring printing search doesn't serve, so only the card can match.
    let served: Uuid = sqlx::query_scalar("SELECT id FROM latest_cards WHERE name = 'Sol Ring'")
        .fetch_one(&pool)
        .await
        .unwrap();
    let unserved = if served == w.sol_a.id() {
        &w.sol_b
    } else {
        &w.sol_a
    };
    own(&pool, &uid, unserved).await;
    own(&pool, &uid, &w.bolt).await;

    let (status, body) = app
        .post(
            SEARCH_CARDS_ROUTE,
            json!({ "owned_only": true }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        names(&body),
        ["Lightning Bolt", "Sol Ring"],
        "Sol Ring counts though the owned printing isn't the one served"
    );
}

#[sqlx::test]
async fn without_owned_only_the_search_is_unchanged(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let w = world(&pool).await;
    let (token, uid) = app.register("owner").await;
    own(&pool, &uid, &w.bolt).await;

    let all = [
        "Krenko, Mob Boss",
        "Lightning Bolt",
        "Llanowar Elves",
        "Sol Ring",
        "Zada, Hedron Grinder",
    ];
    for body in [json!({}), json!({ "owned_only": false })] {
        let (status, reply) = app
            .post(SEARCH_CARDS_ROUTE, body.clone(), Some(&token))
            .await;
        assert_eq!(status, StatusCode::OK, "{reply}");
        assert_eq!(names(&reply), all, "{body}");
    }
}

#[sqlx::test]
async fn another_users_collection_never_counts(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let w = world(&pool).await;
    let (_, owner) = app.register("owner").await;
    own(&pool, &owner, &w.bolt).await;
    own(&pool, &owner, &w.elves).await;
    let (token, _) = app.register("stranger").await;

    let (status, body) = app
        .post(
            SEARCH_CARDS_ROUTE,
            json!({ "owned_only": true }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(names(&body), Vec::<String>::new());
}

#[sqlx::test]
async fn owned_only_narrows_the_decks_add_pile(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let w = world(&pool).await;
    let (token, uid) = app.register("builder").await;
    own(&pool, &uid, &w.elves).await;
    let (status, deck) = app
        .post(
            DECK_ROUTE,
            json!({ "name": "Owned", "format": "commander" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{deck}");
    let route = search_deck_cards_route(deck["id"].as_str().unwrap().parse().unwrap());

    let (status, body) = app
        .post(&route, json!({ "owned_only": true }), Some(&token))
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(names(&body), ["Llanowar Elves"]);
    let (status, body) = app.post(&route, json!({}), Some(&token)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(
        names(&body).len() > 1,
        "without the flag the pile is whole: {body}"
    );
}

#[sqlx::test]
async fn owned_only_narrows_commander_search(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let w = world(&pool).await;
    let (token, uid) = app.register("captain").await;
    own(&pool, &uid, &w.zada).await;

    let (status, body) = app
        .post(
            SEARCH_COMMANDERS_ROUTE,
            json!({ "owned_only": true, "type_line_contains": "Legendary Creature" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(names(&body), ["Zada, Hedron Grinder"]);
    let _ = w.krenko;
}

#[sqlx::test]
async fn owned_only_needs_a_session(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    world(&pool).await;
    let (status, _) = app
        .post(SEARCH_CARDS_ROUTE, json!({ "owned_only": true }), None)
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
