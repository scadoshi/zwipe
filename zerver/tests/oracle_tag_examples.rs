//! The public oracle tag examples route: a reader with no account (zwipe.net's shared deck page)
//! pages through a tag's most played cards. It serves public card data only, a capped page at a
//! time, checks the slug and the offset before any query, and limits each address.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.
#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "a test: helpers unwrap and index, and a panic is the failure report"
)]

mod common;

use axum::http::{Method, StatusCode, header};
use common::{TestApp, card, seed_cards};
use serde_json::Value;
use zwipe_core::domain::card::oracle_tag_examples::{
    EXAMPLES_MAX_OFFSET, EXAMPLES_PAGE_SIZE, OracleTagSlug,
};
use zwipe_core::http::paths::{CARD_ROUTE, oracle_tag_examples_route};

/// The route for `slug` from `offset`, for a slug the test knows is valid.
fn route(slug: &str, offset: u32) -> String {
    let slug: OracleTagSlug = slug.parse().unwrap();
    format!("{}?offset={offset}", oracle_tag_examples_route(&slug))
}

/// The names in a page, in order.
fn names(body: &Value) -> Vec<String> {
    body.as_array()
        .unwrap()
        .iter()
        .map(|c| c["scryfall_data"]["name"].as_str().unwrap().to_owned())
        .collect()
}

/// 27 burn cards ranked 1 to 27, a burn token, and a card with another tag.
async fn seed(pool: &sqlx::PgPool) {
    let mut cards: Vec<_> = (1..=27)
        .map(|rank| {
            card(&format!("Burn {rank:02}"))
                .oracle_tags(&["burn"])
                .edhrec_rank(rank)
        })
        .collect();
    cards.push(
        card("Burn Token")
            .oracle_tags(&["burn"])
            .edhrec_rank(0)
            .token(),
    );
    cards.push(card("Life Card").oracle_tags(&["lifegain"]).edhrec_rank(0));
    seed_cards(pool, &cards).await;
}

#[sqlx::test]
async fn a_reader_with_no_account_pages_through_a_tag_s_most_played_cards(pool: sqlx::PgPool) {
    seed(&pool).await;
    let app = TestApp::new(pool);

    let (status, first) = app.get(&route("burn", 0), None).await;
    assert_eq!(status, StatusCode::OK, "public, no auth: {first}");
    let first_names = names(&first);
    let page = usize::try_from(EXAMPLES_PAGE_SIZE).unwrap();
    assert_eq!(first_names.len(), page, "a page is capped: {first_names:?}");
    let ranked: Vec<String> = (1..=25).map(|rank| format!("Burn {rank:02}")).collect();
    assert_eq!(
        first_names, ranked,
        "most played first, no token, no other tag"
    );

    let (status, second) = app.get(&route("burn", EXAMPLES_PAGE_SIZE), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(names(&second), ["Burn 26", "Burn 27"]);

    let (status, none) = app.get(&route("no-such-tag", 0), None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(names(&none).is_empty(), "{none}");
}

#[sqlx::test]
async fn each_example_is_public_card_data_only(pool: sqlx::PgPool) {
    seed(&pool).await;
    let app = TestApp::new(pool);
    let (_, body) = app.get(&route("burn", 0), None).await;
    for card in body.as_array().unwrap() {
        let mut keys: Vec<&str> = card
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();
        assert_eq!(keys, ["card_profile", "scryfall_data"], "{card}");
    }
}

#[sqlx::test]
async fn a_bad_slug_or_an_offset_past_the_end_is_refused(pool: sqlx::PgPool) {
    seed(&pool).await;
    let app = TestApp::new(pool);
    for slug in ["Burn", "-burn", "burn'1", "b%20urn", &"a".repeat(65)] {
        let path = format!("{CARD_ROUTE}/oracle-tag/{slug}/examples?offset=0");
        let (status, body) = app.get(&path, None).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{slug}: {body}");
    }
    let past = route("burn", EXAMPLES_MAX_OFFSET + 1);
    let (status, body) = app.get(&past, None).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    let (status, _) = app.get(&route("burn", EXAMPLES_MAX_OFFSET), None).await;
    assert_eq!(status, StatusCode::OK, "the last page is served");
}

#[sqlx::test]
async fn the_route_is_limited_per_address(pool: sqlx::PgPool) {
    seed(&pool).await;
    let app = TestApp::new(pool);
    let path = route("burn", 0);
    // A burst of 10 passes; the 11th from the same address is refused.
    for i in 0..10 {
        let (status, _, _) = app.send_bytes(Method::GET, &path).await;
        assert_eq!(status, StatusCode::OK, "request {i}");
    }
    let (status, headers, _) = app.send_bytes(Method::GET, &path).await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert!(headers.contains_key(header::RETRY_AFTER), "{headers:?}");
}
