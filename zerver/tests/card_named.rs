//! A card by name: the slugs the sync stores on every printing.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "a test: a panic is the failure report"
)]

mod common;

use axum::http::{Method, StatusCode, header};
use common::{card, seed_cards};
use uuid::Uuid;
use zwipe::domain::card::models::zervice_metrics::ZerviceMetrics;
use zwipe::domain::card::ports::CardRepository;
use zwipe::domain::card::requests::get_scryfall_data::GetScryfallData;
use zwipe::outbound::sqlx::postgres::Postgres;

/// The stored `(name, name_slug, front_slug)` of every printing, sorted.
async fn slugs(pool: &sqlx::PgPool) -> Vec<(String, Option<String>, Option<String>)> {
    let mut rows: Vec<(String, Option<String>, Option<String>)> =
        sqlx::query_as("SELECT name, name_slug, front_slug FROM scryfall_data")
            .fetch_all(pool)
            .await
            .unwrap();
    rows.sort();
    rows
}

#[sqlx::test]
async fn the_sync_stores_the_name_and_front_face_slugs(pool: sqlx::PgPool) {
    let fixtures = [card("Delver"), card("Vault"), card("Underscores")];
    seed_cards(&pool, &fixtures).await;
    let repo = Postgres::from_pool(pool.clone());
    let mut cards = Vec::new();
    for (fixture, name) in fixtures.iter().zip([
        "Delver of Secrets // Insectile Aberration",
        "Lim-Dûl's Vault",
        "_____",
    ]) {
        let mut data = repo
            .fetch_scryfall_data(&GetScryfallData::new(&fixture.id().to_string()).unwrap())
            .await
            .unwrap();
        data.name = name.to_owned();
        cards.push(data);
    }
    let (single, rest) = cards.split_first().unwrap();

    let _ = repo.upsert(single).await.unwrap();
    let mut metrics = ZerviceMetrics::new();
    let _ = repo.batch_upsert(rest, 100, &mut metrics).await.unwrap();

    assert_eq!(
        slugs(&pool).await,
        vec![
            (
                "Delver of Secrets // Insectile Aberration".to_owned(),
                Some("delver-of-secrets-insectile-aberration".to_owned()),
                Some("delver-of-secrets".to_owned())
            ),
            (
                "Lim-Dûl's Vault".to_owned(),
                Some("lim-dul-s-vault".to_owned()),
                None
            ),
            ("_____".to_owned(), None, None),
        ]
    );
}

/// The names `GET /api/card/named/{slug}` answers, in order, and its `Cache-Control`.
async fn named(app: &common::TestApp, slug: &str) -> (Vec<String>, Option<String>) {
    let (status, headers, body) = app
        .send_raw(
            Method::GET,
            &format!("/api/card/named/{slug}"),
            None,
            None,
            &[],
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{slug}: {body}");
    let names = body
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["scryfall_data"]["name"].as_str().unwrap().to_owned())
        .collect();
    let cache = headers
        .get(header::CACHE_CONTROL)
        .map(|v| v.to_str().unwrap().to_owned());
    (names, cache)
}

#[sqlx::test]
async fn a_full_name_match_ranks_before_a_front_face(pool: sqlx::PgPool) {
    seed_cards(
        &pool,
        &[
            card("Bind // Liberate").oracle(Some(Uuid::from_u128(0xb1))),
            card("Bind").oracle(Some(Uuid::from_u128(0xb2))),
        ],
    )
    .await;
    let app = common::TestApp::new(pool);
    let (names, cache) = named(&app, "bind").await;
    assert_eq!(names, ["Bind", "Bind // Liberate"]);
    assert_eq!(cache.as_deref(), Some("public, max-age=3600"));
    assert_eq!(named(&app, "bind-liberate").await.0, ["Bind // Liberate"]);
}

#[sqlx::test]
async fn a_multi_faced_card_answers_to_its_front_face(pool: sqlx::PgPool) {
    seed_cards(&pool, &[card("Delver of Secrets // Insectile Aberration")]).await;
    let app = common::TestApp::new(pool);
    let want = ["Delver of Secrets // Insectile Aberration"];
    assert_eq!(named(&app, "delver-of-secrets").await.0, want);
    assert_eq!(
        named(&app, "delver-of-secrets-insectile-aberration")
            .await
            .0,
        want
    );
}

#[sqlx::test]
async fn a_shared_slug_answers_every_card_playable_first_one_printing_each(pool: sqlx::PgPool) {
    let growth = Uuid::from_u128(0x6a);
    seed_cards(
        &pool,
        &[
            card("Rampant Growth")
                .oracle(Some(Uuid::from_u128(0x6b)))
                .token(),
            card("Rampant, Growth").oracle(Some(Uuid::from_u128(0x6c))),
            card("Rampant Growth")
                .oracle(Some(growth))
                .set("m10", "Magic 2010"),
            card("Rampant Growth")
                .oracle(Some(growth))
                .set("m11", "Magic 2011"),
        ],
    )
    .await;
    let app = common::TestApp::new(pool);
    let (names, _) = named(&app, "rampant-growth").await;
    assert_eq!(names.len(), 3, "one per card: {names:?}");
    assert_eq!(
        names.last().map(String::as_str),
        Some("Rampant Growth"),
        "the token last: {names:?}"
    );
    assert!(names.contains(&"Rampant, Growth".to_owned()), "{names:?}");
}

#[sqlx::test]
async fn no_such_card_and_a_malformed_slug_answer_an_empty_list(pool: sqlx::PgPool) {
    seed_cards(&pool, &[card("Sol Ring")]).await;
    let app = common::TestApp::new(pool);
    assert_eq!(
        named(&app, "no-such-card").await.0,
        Vec::<String>::new(),
        "no-such-card"
    );
    assert_eq!(
        named(&app, "Sol-Ring").await.0,
        Vec::<String>::new(),
        "Sol-Ring"
    );
    assert_eq!(
        named(&app, "sol--ring").await.0,
        Vec::<String>::new(),
        "sol--ring"
    );
}

#[sqlx::test]
async fn one_slug_answers_at_most_fifty_cards(pool: sqlx::PgPool) {
    // Fifty tokens and one playable card, so the cap keeps the playable one.
    let mut cards: Vec<_> = (0u128..50)
        .map(|i| {
            card("Goblin")
                .oracle(Some(Uuid::from_u128(
                    0x60b0_0000_u128.checked_add(i).unwrap(),
                )))
                .token()
        })
        .collect();
    cards.push(card("Goblin").oracle(Some(Uuid::from_u128(0x60b1_0000))));
    seed_cards(&pool, &cards).await;
    let app = common::TestApp::new(pool);
    let (names, _) = named(&app, "goblin").await;
    assert_eq!(names.len(), 50);
    let (status, body) = app
        .send(Method::GET, "/api/card/named/goblin", None, None)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body[0]["scryfall_data"]["layout"], "normal",
        "the playable card is kept: {body}"
    );
}

#[sqlx::test]
async fn reads_go_on_while_the_slug_indexes_build(pool: sqlx::PgPool) {
    seed_cards(&pool, &[card("Sol Ring")]).await;
    sqlx::query("DROP INDEX idx_scryfall_data_name_slug, idx_scryfall_data_front_slug")
        .execute(&pool)
        .await
        .unwrap();
    // The index migration, held open as a slow build would hold it.
    let mut building = pool.begin().await.unwrap();
    sqlx::raw_sql(include_str!(
        "../migrations/20261119100050_card_slugs_index.sql"
    ))
    .execute(&mut *building)
    .await
    .unwrap();

    let mut reader = pool.acquire().await.unwrap();
    sqlx::query("SET statement_timeout = '1s'")
        .execute(&mut *reader)
        .await
        .unwrap();
    let read: Result<i64, _> = sqlx::query_scalar("SELECT count(*) FROM scryfall_data")
        .fetch_one(&mut *reader)
        .await;
    building.commit().await.unwrap();
    assert_eq!(read.unwrap(), 1, "a search reads while the indexes build");
}
