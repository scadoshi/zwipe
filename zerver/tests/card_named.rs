//! A card by name: the slugs the sync stores on every printing.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![expect(clippy::unwrap_used, reason = "a test: a panic is the failure report")]

mod common;

use common::{card, seed_cards};
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
