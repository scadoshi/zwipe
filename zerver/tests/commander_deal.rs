//! `POST /api/commander/deal`: one page of the merged commander ranking over every commander a
//! filter matches. Popularity orders it for a user with nothing to go on, a filter outside the
//! user's colors still answers, the user's led commanders, maybeboard, dismissals and the
//! request's own exclusions stay out, pages split one order, a saved commander lifts its
//! neighbors, and one identity past its first few sorts after the rest.
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
use zwipe_core::domain::card::scryfall_data::colors::{Color, Colors};
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::http::paths::{
    COMMANDER_DEAL_ROUTE, COMMANDER_DISMISSALS_ROUTE, DECK_ROUTE,
    add_commander_maybeboard_card_route, deck_route,
};

/// A commander-legal legendary creature of color identity `identity`, ranked `rank` on EDHREC.
fn commander(name: &str, identity: &str, rank: i32) -> CardFixture {
    card(name)
        .mono(identity)
        .type_line("Legendary Creature — Human")
        .commander_legal()
        .edhrec_rank(rank)
}

/// Writes one commander's precomputed tag profile, as the synergy worker does.
async fn seed_profile(pool: &sqlx::PgPool, oracle_id: Uuid, profile: Value) {
    sqlx::query("INSERT INTO commander_tag_profiles (oracle_id, profile) VALUES ($1, $2)")
        .bind(oracle_id)
        .bind(sqlx::types::Json(profile))
        .execute(pool)
        .await
        .unwrap();
}

/// A signed-in, verified user named `name`.
async fn user(app: &TestApp, name: &str) -> String {
    let (token, user_id) = app.register(name).await;
    app.verify_email(&user_id).await;
    token
}

/// One deal page for `body`, with its status checked.
async fn deal(app: &TestApp, token: &str, body: Value) -> Value {
    let (status, page) = app.post(COMMANDER_DEAL_ROUTE, body, Some(token)).await;
    assert_eq!(status, StatusCode::OK, "deal: {page}");
    page
}

/// The page's commander names, in order.
fn names(page: &Value) -> Vec<String> {
    page["suggestions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| {
            s["card"]["scryfall_data"]["name"]
                .as_str()
                .unwrap()
                .to_owned()
        })
        .collect()
}

/// A filter of exactly `colors` as the color identity.
fn identity_filter(colors: &[Color]) -> Value {
    let mut filter = CardQueryBuilder::default();
    filter.set_color_identity_equals(Colors::from(colors.to_vec()));
    serde_json::to_value(filter.build().unwrap()).unwrap()
}

#[sqlx::test]
async fn a_user_with_nothing_to_go_on_gets_popularity_order_and_only_commanders(
    pool: sqlx::PgPool,
) {
    let app = TestApp::new(pool.clone());
    let token = user(&app, "fresh").await;
    seed_cards(
        &pool,
        &[
            commander("Third", "W", 300),
            commander("First", "U", 100),
            commander("Second", "B", 200),
            card("Not A Commander").mono("R").edhrec_rank(1),
        ],
    )
    .await;

    let page = deal(&app, &token, json!({})).await;
    assert_eq!(names(&page), ["First", "Second", "Third"]);
    assert_eq!(page["ranked_by"], "popularity");
}

#[sqlx::test]
async fn a_filter_outside_the_user_s_colors_still_answers(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let token = user(&app, "redplayer").await;
    let leader = commander("Red Leader", "R", 10);
    let leader_id = leader.id();
    seed_cards(
        &pool,
        &[
            leader,
            commander("Red Peer", "R", 20),
            commander("Esper Lord", "WUB", 900),
        ],
    )
    .await;
    let (status, deck) = app
        .post(
            DECK_ROUTE,
            json!({ "name": "Red", "format": "commander" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "create: {deck}");
    let deck_id: Uuid = deck["id"].as_str().unwrap().parse().unwrap();
    let (status, body) = app
        .patch(
            &deck_route(deck_id),
            json!({ "commander_id": leader_id.to_string() }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "set commander: {body}");

    let page = deal(
        &app,
        &token,
        json!({ "filter": identity_filter(&[Color::White, Color::Blue, Color::Black]) }),
    )
    .await;
    assert_eq!(names(&page), ["Esper Lord"]);
}

#[sqlx::test]
async fn led_saved_dismissed_and_dealt_commanders_stay_out(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let token = user(&app, "excluder").await;
    let led = commander("Led", "W", 1);
    let saved = commander("Saved", "U", 2);
    let dismissed = commander("Dismissed", "B", 3);
    let dealt = commander("Dealt", "R", 4);
    let open = commander("Open", "G", 5);
    let (led_id, saved_oid, dismissed_oid, dealt_oid) = (
        led.id(),
        saved.oracle_id().unwrap(),
        dismissed.oracle_id().unwrap(),
        dealt.oracle_id().unwrap(),
    );
    seed_cards(&pool, &[led, saved, dismissed, dealt, open]).await;
    let (status, deck) = app
        .post(
            DECK_ROUTE,
            json!({ "name": "Led", "format": "commander" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "create: {deck}");
    let deck_id: Uuid = deck["id"].as_str().unwrap().parse().unwrap();
    let (status, body) = app
        .patch(
            &deck_route(deck_id),
            json!({ "commander_id": led_id.to_string() }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "set commander: {body}");
    let (status, body) = app
        .post(
            &add_commander_maybeboard_card_route(saved_oid),
            json!({}),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT, "maybeboard: {body}");
    let (status, body) = app
        .post(
            COMMANDER_DISMISSALS_ROUTE,
            json!({ "oracle_id": dismissed_oid }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT, "dismiss: {body}");

    let suggested = deal(&app, &token, json!({ "exclude": [dealt_oid] })).await;
    assert_eq!(
        names(&suggested),
        ["Open"],
        "Suggested leaves the maybeboard out"
    );
    let deck_creation = deal(
        &app,
        &token,
        json!({ "exclude": [dealt_oid], "keep_maybeboard": true }),
    )
    .await;
    assert_eq!(
        names(&deck_creation),
        ["Saved", "Open"],
        "deck creation keeps the maybeboard; leads, dismissals and dealt ids stay out"
    );
}

#[sqlx::test]
async fn pages_split_one_order_without_overlap(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let token = user(&app, "pager").await;
    let identities = ["W", "U", "B", "R", "G", "WU", "UB"];
    let cards: Vec<CardFixture> = identities
        .iter()
        .zip(1..)
        .map(|(identity, rank)| commander(&format!("C{rank}"), identity, rank))
        .collect();
    seed_cards(&pool, &cards).await;

    let whole = names(&deal(&app, &token, json!({ "limit": 7 })).await);
    let mut paged = Vec::new();
    for offset in [0, 3, 6] {
        paged.extend(names(
            &deal(&app, &token, json!({ "offset": offset, "limit": 3 })).await,
        ));
    }
    assert_eq!(paged, whole, "three pages are the whole list in order");
    assert_eq!(whole.len(), 7);
}

#[sqlx::test]
async fn a_saved_commander_lifts_its_neighbors_over_fame(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let token = user(&app, "saver").await;
    let saved = commander("Saved Tokens", "W", 500);
    let neighbor = commander("Token Neighbor", "G", 400);
    let famous = commander("Famous Stranger", "U", 1);
    let (saved_oid, neighbor_oid, famous_oid) = (
        saved.oracle_id().unwrap(),
        neighbor.oracle_id().unwrap(),
        famous.oracle_id().unwrap(),
    );
    seed_cards(&pool, &[saved, neighbor, famous]).await;
    seed_profile(&pool, saved_oid, json!({ "tokens": 1.0 })).await;
    seed_profile(&pool, neighbor_oid, json!({ "tokens": 1.0 })).await;
    seed_profile(&pool, famous_oid, json!({ "lifegain": 1.0 })).await;

    let before = deal(&app, &token, json!({})).await;
    assert_eq!(
        names(&before).first().map(String::as_str),
        Some("Famous Stranger")
    );
    assert_eq!(before["ranked_by"], "popularity");

    let (status, body) = app
        .post(
            &add_commander_maybeboard_card_route(saved_oid),
            json!({}),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT, "maybeboard: {body}");
    let after = deal(&app, &token, json!({})).await;
    assert_eq!(names(&after), ["Token Neighbor", "Famous Stranger"]);
    assert_eq!(
        after["ranked_by"], "playstyle",
        "a saved profile is something to go on"
    );
}

#[sqlx::test]
async fn one_identity_past_its_first_four_sorts_after_the_rest_unless_filtered_to_it(
    pool: sqlx::PgPool,
) {
    let app = TestApp::new(pool.clone());
    let token = user(&app, "spreader").await;
    let mut cards: Vec<CardFixture> = (1..=6)
        .map(|rank| commander(&format!("Red {rank}"), "R", rank))
        .collect();
    cards.push(commander("Green", "G", 100));
    seed_cards(&pool, &cards).await;

    let spread = names(&deal(&app, &token, json!({ "limit": 7 })).await);
    assert_eq!(
        spread,
        [
            "Red 1", "Red 2", "Red 3", "Red 4", "Green", "Red 5", "Red 6"
        ]
    );
    let only_red = names(
        &deal(
            &app,
            &token,
            json!({ "limit": 7, "filter": identity_filter(&[Color::Red]) }),
        )
        .await,
    );
    assert_eq!(
        only_red,
        ["Red 1", "Red 2", "Red 3", "Red 4", "Red 5", "Red 6"],
        "a named identity has nothing to spread across"
    );
}

#[sqlx::test]
async fn played_colors_break_a_tie_and_saved_colors_count_toward_a_save_s_fit(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let token = user(&app, "colorful").await;
    let leader = commander("Red Leader", "R", 1);
    let leader_id = leader.id();
    let saved = commander("Saved Green", "G", 2);
    let saved_oid = saved.oracle_id().unwrap();
    seed_cards(
        &pool,
        &[
            leader,
            saved,
            commander("Blue Peer", "U", 50),
            commander("Red Peer", "R", 50),
            commander("Green Peer", "G", 50),
            commander("Black Peer", "B", 50),
        ],
    )
    .await;
    let (status, deck) = app
        .post(
            DECK_ROUTE,
            json!({ "name": "Red", "format": "commander" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "create: {deck}");
    let deck_id: Uuid = deck["id"].as_str().unwrap().parse().unwrap();
    let (status, body) = app
        .patch(
            &deck_route(deck_id),
            json!({ "commander_id": leader_id.to_string() }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "set commander: {body}");
    let (status, body) = app
        .post(
            &add_commander_maybeboard_card_route(saved_oid),
            json!({}),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT, "maybeboard: {body}");

    let mut dealt = names(&deal(&app, &token, json!({})).await);
    assert_eq!(
        dealt.get(..2),
        Some(&["Red Peer".to_owned(), "Green Peer".to_owned()][..]),
        "the deck's red leads, then the saved commander's green"
    );
    let mut tied = dealt.split_off(2);
    tied.sort();
    assert_eq!(
        tied,
        ["Black Peer", "Blue Peer"],
        "the rest tie on popularity"
    );
}

#[sqlx::test]
async fn the_filter_s_owned_only_deals_the_user_s_collection_in_ranking_order(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, user_id) = app.register("collector").await;
    app.verify_email(&user_id).await;
    let owned_late = commander("Owned Late", "W", 40);
    let owned_early = commander("Owned Early", "U", 10);
    let printings = [
        (owned_late.id(), owned_late.oracle_id().unwrap()),
        (owned_early.id(), owned_early.oracle_id().unwrap()),
    ];
    seed_cards(
        &pool,
        &[
            owned_late,
            owned_early,
            commander("Unowned Famous", "B", 1),
            commander("Unowned Other", "R", 20),
        ],
    )
    .await;
    for (scryfall_data_id, oracle_id) in printings {
        sqlx::query(
            "INSERT INTO collection_cards (user_id, scryfall_data_id, oracle_id, finish, quantity) \
             VALUES ($1, $2, $3, 'nonfoil', 1)",
        )
        .bind(user_id.parse::<Uuid>().unwrap())
        .bind(scryfall_data_id)
        .bind(oracle_id)
        .execute(&pool)
        .await
        .unwrap();
    }

    let mut owned_filter = CardQueryBuilder::default();
    owned_filter.set_owned_only(true);
    let owned_filter = serde_json::to_value(owned_filter.build().unwrap()).unwrap();
    let owned = deal(&app, &token, json!({ "filter": owned_filter })).await;
    assert_eq!(names(&owned), ["Owned Early", "Owned Late"]);
    let every = deal(&app, &token, json!({})).await;
    assert_eq!(
        names(&every),
        [
            "Unowned Famous",
            "Owned Early",
            "Unowned Other",
            "Owned Late"
        ],
        "off by default"
    );
}
