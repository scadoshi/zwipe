//! `POST /api/card/lookup`, the card search a signed-out caller may make: it answers at most 30
//! cards, one per card, takes no paging, sorting or other search field, checks every value it
//! does take, reads its chips as the filter does, applies a signed-in caller's Universes Beyond
//! setting, and limits each address.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "a test: helpers outside #[test] fns unwrap and index, and a panic is the failure report"
)]

mod common;

use std::net::SocketAddr;

use axum::http::{Method, StatusCode, header};
use common::{TestApp, card, seed_cards};
use serde_json::{Value, json};
use uuid::Uuid;
use zwipe_core::domain::card::models::lookup::LOOKUP_LIMIT;
use zwipe_core::http::paths::{CARD_LOOKUP_ROUTE, SETTINGS_ROUTE};

/// The names in a lookup's answer, sorted.
fn names(results: &Value) -> Vec<String> {
    let mut names: Vec<String> = results
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["scryfall_data"]["name"].as_str().unwrap().to_owned())
        .collect();
    names.sort();
    names
}

#[sqlx::test]
async fn a_signed_out_lookup_answers_at_most_30_cards_one_per_card(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let reprinted = Some(Uuid::new_v4());
    let mut cards: Vec<_> = (2..=35)
        .map(|n| card(&format!("Goblin {n:02}")).mono("R"))
        .collect();
    cards.push(card("Goblin 01").mono("R").oracle(reprinted));
    cards.push(
        card("Goblin 01")
            .mono("R")
            .oracle(reprinted)
            .set("m10", "Magic 2010"),
    );
    seed_cards(&pool, &cards).await;

    let (status, results) = app
        .post(CARD_LOOKUP_ROUTE, json!({ "name": "goblin" }), None)
        .await;
    assert_eq!(status, StatusCode::OK, "{results}");
    let found = names(&results);
    assert_eq!(
        found.len(),
        usize::try_from(LOOKUP_LIMIT).unwrap(),
        "{found:?}"
    );
    let mut unique = found.clone();
    unique.dedup();
    assert_eq!(unique, found, "one printing per card");
}

#[sqlx::test]
async fn paging_sorting_and_every_other_search_field_are_refused(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    seed_cards(&pool, &[card("Lightning Bolt").mono("R")]).await;
    for (field, value) in [
        ("limit", json!(1000)),
        ("offset", json!(30)),
        ("order_by", json!("Name")),
        ("direction", json!("descending")),
        ("synergy", json!(true)),
        ("include_skipped", json!(true)),
        ("oracle_text_contains", json!("damage")),
    ] {
        let mut body = json!({ "name": "bolt" });
        body[field] = value;
        let (status, answer) = app.post(CARD_LOOKUP_ROUTE, body, None).await;
        assert_eq!(
            status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "{field}: {answer}"
        );
    }
}

#[sqlx::test]
async fn every_value_is_checked(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    for body in [
        json!({}),
        json!({ "name": "" }),
        json!({ "name": "a".repeat(101) }),
        json!({ "name": "bolt\u{0}" }),
        json!({ "name": "x", "types": ["A", "B", "C", "D", "E", "F"] }),
        json!({ "name": "x", "types": ["Gob%lin"] }),
        json!({ "name": "x", "colors": ["W", "U", "B", "R", "G", "W"] }),
        json!({ "name": "x", "colors": ["P"] }),
        json!({ "name": "x", "card_roles": ["teleport"] }),
        json!({ "name": "x", "card_roles": ["ramp", "draw", "wipe", "burn", "mill", "pump"] }),
    ] {
        let (status, answer) = app.post(CARD_LOOKUP_ROUTE, body.clone(), None).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}: {answer}");
    }
}

#[sqlx::test]
async fn the_chips_mean_what_the_filter_s_chips_mean(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    seed_cards(
        &pool,
        &[
            card("Goblin Guide")
                .mono("R")
                .type_line("Creature — Goblin")
                .categories(&["aggression"]),
            card("Llanowar Elves")
                .mono("G")
                .type_line("Creature — Elf Druid")
                .categories(&["ramp"]),
            card("Lightning Bolt")
                .mono("R")
                .type_line("Instant")
                .categories(&["burn"]),
            card("Manamorphose")
                .colors("RG")
                .color_identity("RG")
                .type_line("Instant")
                .categories(&["draw"]),
        ],
    )
    .await;
    let lookup = |body: Value| {
        let app = &app;
        async move {
            let (status, results) = app.post(CARD_LOOKUP_ROUTE, body, None).await;
            assert_eq!(status, StatusCode::OK, "{results}");
            names(&results)
        }
    };

    assert_eq!(
        lookup(json!({ "name": "", "types": ["Goblin", "Instant"] })).await,
        ["Goblin Guide", "Lightning Bolt", "Manamorphose"],
        "types match any of them"
    );
    assert_eq!(
        lookup(json!({ "name": "", "colors": ["R"] })).await,
        ["Goblin Guide", "Lightning Bolt"],
        "colors are the color identity, exactly"
    );
    assert_eq!(
        lookup(json!({ "name": "", "colors": ["G", "R"] })).await,
        ["Manamorphose"]
    );
    assert_eq!(
        lookup(json!({ "name": "", "card_roles": ["ramp", "draw"] })).await,
        ["Llanowar Elves", "Manamorphose"],
        "roles match any of them"
    );
    assert_eq!(
        lookup(json!({ "name": "l", "types": ["Instant"], "colors": ["R"] })).await,
        ["Lightning Bolt"],
        "the name and the chips all hold"
    );
}

#[sqlx::test]
async fn a_token_applies_the_universes_beyond_setting_and_a_bad_one_is_refused(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    seed_cards(
        &pool,
        &[card("Sonic the Hedgehog")
            .mono("R")
            .set("sld", "Secret Lair Drop")
            .stamp("triangle")],
    )
    .await;
    let body = json!({ "name": "sonic" });

    let (status, results) = app.post(CARD_LOOKUP_ROUTE, body.clone(), None).await;
    assert_eq!(status, StatusCode::OK, "{results}");
    assert_eq!(
        names(&results),
        ["Sonic the Hedgehog"],
        "signed out excludes nothing"
    );

    let (token, _) = app.register("lookupub").await;
    let (status, _) = app
        .patch(
            SETTINGS_ROUTE,
            json!({ "exclude_universes_beyond": true }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let (status, results) = app
        .post(CARD_LOOKUP_ROUTE, body.clone(), Some(&token))
        .await;
    assert_eq!(status, StatusCode::OK, "{results}");
    assert!(names(&results).is_empty(), "the setting applies: {results}");

    let (status, _) = app
        .post(CARD_LOOKUP_ROUTE, body.clone(), Some("not-a-token"))
        .await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "a bad token isn't read as signed out"
    );
    let (status, _, _) = app
        .send_raw(
            Method::POST,
            CARD_LOOKUP_ROUTE,
            Some(body),
            None,
            &[(header::AUTHORIZATION, "Basic Zm9vOmJhcg==".to_owned())],
        )
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "nor is another scheme");
}

#[sqlx::test]
async fn the_route_is_limited_per_address(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    seed_cards(&pool, &[card("Lightning Bolt").mono("R")]).await;
    let body = json!({ "name": "bolt" });
    // A burst of 20 passes; the 21st from the same address is refused.
    for i in 0..20 {
        let (status, _) = app.post(CARD_LOOKUP_ROUTE, body.clone(), None).await;
        assert_eq!(status, StatusCode::OK, "request {i}");
    }
    let (status, headers, _) = app
        .send_raw(Method::POST, CARD_LOOKUP_ROUTE, Some(body), None, &[])
        .await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert!(headers.contains_key(header::RETRY_AFTER), "{headers:?}");
    assert_eq!(
        headers
            .get(header::CACHE_CONTROL)
            .and_then(|v| v.to_str().ok()),
        Some("private, no-store"),
        "a refusal isn't cached either"
    );
}

#[sqlx::test]
async fn a_body_over_4_kb_is_refused_before_it_is_read(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let padded = json!({ "name": "bolt", "pad": "x".repeat(5000) });
    let (status, _, _) = app
        .send_raw(Method::POST, CARD_LOOKUP_ROUTE, Some(padded), None, &[])
        .await;
    assert_eq!(
        status,
        StatusCode::PAYLOAD_TOO_LARGE,
        "a lookup body is at most about 1 KB"
    );
    let (status, _) = app
        .post(CARD_LOOKUP_ROUTE, json!({ "name": "bolt" }), None)
        .await;
    assert_eq!(status, StatusCode::OK, "a real lookup still fits");
}

#[sqlx::test]
async fn a_name_of_only_punctuation_is_refused_rather_than_searching_everything(
    pool: sqlx::PgPool,
) {
    let app = TestApp::new(pool.clone());
    seed_cards(&pool, &[card("Lightning Bolt").mono("R")]).await;
    let (status, _) = app
        .post(CARD_LOOKUP_ROUTE, json!({ "name": "%%%" }), None)
        .await;
    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "it would match every card"
    );
}

#[sqlx::test]
async fn a_name_matches_however_its_punctuation_and_spaces_are_typed(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    seed_cards(
        &pool,
        &[
            card("Fire // Ice").mono("R"),
            card("Bounty: Miron Tillas Jr. // Wanted!").mono("B"),
            card("Lightning Bolt").mono("R"),
        ],
    )
    .await;
    for (typed, want) in [
        ("fire ice", "Fire // Ice"),
        ("fire // ice", "Fire // Ice"),
        ("fire  ice", "Fire // Ice"),
        ("bounty miron", "Bounty: Miron Tillas Jr. // Wanted!"),
        ("bounty: miron", "Bounty: Miron Tillas Jr. // Wanted!"),
        ("tillas jr wanted", "Bounty: Miron Tillas Jr. // Wanted!"),
    ] {
        let (status, results) = app
            .post(CARD_LOOKUP_ROUTE, json!({ "name": typed }), None)
            .await;
        assert_eq!(status, StatusCode::OK, "{typed:?}: {results}");
        assert_eq!(names(&results), [want], "{typed:?}");
    }
}

#[sqlx::test]
async fn the_whole_name_leads_then_a_name_that_starts_with_it_then_a_word_that_does(
    pool: sqlx::PgPool,
) {
    let app = TestApp::new(pool.clone());
    let mut cards: Vec<_> = (1..=35)
        .map(|n| card(&format!("Consolidate {n:02}")).mono("W"))
        .collect();
    cards.extend([
        card("Gift of Solace").mono("W"),
        card("Solemn Simulacrum").mono("W"),
        card("Sol Ring").mono("W"),
        card("Sol").mono("W"),
    ]);
    seed_cards(&pool, &cards).await;

    let (status, results) = app
        .post(CARD_LOOKUP_ROUTE, json!({ "name": "sol" }), None)
        .await;
    assert_eq!(status, StatusCode::OK, "{results}");
    let in_order: Vec<&str> = results
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["scryfall_data"]["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        in_order.get(..4),
        Some(&["Sol", "Sol Ring", "Solemn Simulacrum", "Gift of Solace"][..]),
        "{in_order:?}"
    );
    assert_eq!(in_order.len(), usize::try_from(LOOKUP_LIMIT).unwrap());
}

#[sqlx::test]
async fn no_answer_is_cached_since_a_signed_in_one_follows_the_user_s_setting(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    seed_cards(&pool, &[card("Lightning Bolt").mono("R")]).await;
    for (body, want) in [
        (json!({ "name": "bolt" }), StatusCode::OK),
        (json!({ "name": "%%%" }), StatusCode::UNPROCESSABLE_ENTITY),
        (
            json!({ "name": "bolt", "pad": "x".repeat(5000) }),
            StatusCode::PAYLOAD_TOO_LARGE,
        ),
    ] {
        let (status, headers, _) = app
            .send_raw(Method::POST, CARD_LOOKUP_ROUTE, Some(body), None, &[])
            .await;
        assert_eq!(status, want);
        assert_eq!(
            headers
                .get(header::CACHE_CONTROL)
                .and_then(|v| v.to_str().ok()),
            Some("private, no-store"),
            "{status}"
        );
    }
}

/// An IPv6 client counts by its /64, which a household shares, so it gets twice IPv4's burst.
#[sqlx::test]
async fn an_ipv6_64_gets_twice_the_burst(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    seed_cards(&pool, &[card("Lightning Bolt").mono("R")]).await;
    let body = json!({ "name": "bolt" });
    let mut passed = 0_u32;
    for n in 1..=60_u32 {
        let peer = SocketAddr::new(format!("2001:db8:1:2::{n:x}").parse().unwrap(), 50001);
        let (status, _) = app
            .send_from(
                peer,
                Method::POST,
                CARD_LOOKUP_ROUTE,
                Some(body.clone()),
                None,
            )
            .await;
        if status == StatusCode::TOO_MANY_REQUESTS {
            break;
        }
        assert_eq!(status, StatusCode::OK, "request {n}");
        passed = n;
    }
    assert_eq!(passed, 40, "twice IPv4's 20, one bucket for the whole /64");
}

/// A browser reads a 429's `Retry-After` only when CORS exposes it, so the web's search can wait
/// the server's time instead of guessing.
#[sqlx::test]
async fn a_browser_can_read_a_429_s_retry_after(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    seed_cards(&pool, &[card("Lightning Bolt").mono("R")]).await;
    let body = json!({ "name": "bolt" });
    let origin = [(header::ORIGIN, common::TEST_ORIGIN.to_owned())];
    for _ in 0..20 {
        let _ = app
            .send_raw(
                Method::POST,
                CARD_LOOKUP_ROUTE,
                Some(body.clone()),
                None,
                &origin,
            )
            .await;
    }
    let (status, headers, _) = app
        .send_raw(Method::POST, CARD_LOOKUP_ROUTE, Some(body), None, &origin)
        .await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert!(headers.contains_key(header::RETRY_AFTER), "{headers:?}");
    let exposed = headers
        .get(header::ACCESS_CONTROL_EXPOSE_HEADERS)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    assert!(
        exposed.split(',').any(|name| name.trim() == "retry-after"),
        "exposed: {exposed:?}"
    );
}
