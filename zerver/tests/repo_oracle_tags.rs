//! Repo-level Oracle Tags ingest tests. Constructs `Postgres { pool }` and calls
//! `CardRepository::sync_oracle_tags` directly, asserting the catalog + card
//! correlation land, that re-syncing fully replaces scryfall rows idempotently,
//! and that non-scryfall (heuristic) correlations survive a re-sync.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use uuid::Uuid;

use zwipe::{
    domain::card::ports::CardRepository,
    inbound::external::scryfall::oracle_tag::{OracleTag, Tagging},
    outbound::sqlx::postgres::Postgres,
};

fn tag(id: u128, slug: &str, description: Option<&str>, cards: &[Uuid]) -> OracleTag {
    OracleTag {
        id: Uuid::from_u128(id),
        slug: slug.to_string(),
        label: slug.to_string(),
        description: description.map(str::to_string),
        parent_ids: vec![],
        aliases: vec![],
        taggings: cards
            .iter()
            .map(|&c| Tagging {
                oracle_id: Some(c),
            })
            .collect(),
    }
}

async fn count(pool: &sqlx::PgPool, sql: &str) -> i64 {
    sqlx::query_scalar(sql).fetch_one(pool).await.unwrap()
}

#[sqlx::test]
async fn sync_populates_catalog_and_correlations(pool: sqlx::PgPool) {
    let repo = Postgres { pool: pool.clone() };
    let card_a = Uuid::from_u128(0xA);
    let card_b = Uuid::from_u128(0xB);
    let tags = vec![
        tag(0x1, "removal", Some("Removes stuff"), &[card_a, card_b]),
        tag(0x2, "ramp", None, &[card_a]),
    ];

    let (catalog, correlations) = repo.sync_oracle_tags(&tags).await.unwrap();
    assert_eq!(catalog, 2);
    assert_eq!(correlations, 3);

    // null description round-trips as NULL
    let desc: Option<String> =
        sqlx::query_scalar("SELECT description FROM oracle_tags WHERE slug = 'ramp'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(desc.is_none());

    // card_a carries both oracle tags
    let a_tags: Vec<String> = sqlx::query_scalar(
        "SELECT oracle_tag FROM card_oracle_tags WHERE oracle_id = $1 ORDER BY oracle_tag",
    )
    .bind(card_a)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(a_tags, vec!["ramp".to_string(), "removal".to_string()]);

    // source defaults to 'scryfall'
    let source: String = sqlx::query_scalar(
        "SELECT source FROM card_oracle_tags WHERE oracle_id = $1 AND oracle_tag = 'ramp'",
    )
    .bind(card_a)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(source, "scryfall");
}

#[sqlx::test]
async fn re_sync_fully_replaces_scryfall_rows(pool: sqlx::PgPool) {
    let repo = Postgres { pool: pool.clone() };
    let card = Uuid::from_u128(0xC);

    // First sync: two oracle tags on the card.
    repo.sync_oracle_tags(&[
        tag(0x1, "removal", None, &[card]),
        tag(0x2, "ramp", None, &[card]),
    ])
    .await
    .unwrap();
    assert_eq!(count(&pool, "SELECT count(*) FROM card_oracle_tags").await, 2);

    // Second sync drops 'ramp' and re-sends 'removal' — full replace, no dupes.
    let (catalog, correlations) = repo
        .sync_oracle_tags(&[tag(0x1, "removal", None, &[card])])
        .await
        .unwrap();
    assert_eq!(catalog, 1);
    assert_eq!(correlations, 1);
    assert_eq!(count(&pool, "SELECT count(*) FROM card_oracle_tags").await, 1);
    assert_eq!(count(&pool, "SELECT count(*) FROM oracle_tags").await, 1);
    assert_eq!(
        count(
            &pool,
            "SELECT count(*) FROM card_oracle_tags WHERE oracle_tag = 'removal'"
        )
        .await,
        1
    );
    assert_eq!(
        count(
            &pool,
            "SELECT count(*) FROM card_oracle_tags WHERE oracle_tag = 'ramp'"
        )
        .await,
        0
    );
}

#[sqlx::test]
async fn re_sync_preserves_heuristic_rows(pool: sqlx::PgPool) {
    let repo = Postgres { pool: pool.clone() };
    let card = Uuid::from_u128(0xD);

    // Seed a heuristic-sourced correlation (as a later phase would).
    sqlx::query(
        "INSERT INTO card_oracle_tags (oracle_id, oracle_tag, source) VALUES ($1, 'ramp', 'heuristic')",
    )
    .bind(card)
    .execute(&pool)
    .await
    .unwrap();

    // A scryfall sync of a different tag must leave the heuristic row intact.
    repo.sync_oracle_tags(&[tag(0x1, "removal", None, &[card])])
        .await
        .unwrap();

    assert_eq!(
        count(
            &pool,
            "SELECT count(*) FROM card_oracle_tags WHERE source = 'heuristic'"
        )
        .await,
        1
    );
    assert_eq!(
        count(
            &pool,
            "SELECT count(*) FROM card_oracle_tags WHERE source = 'scryfall'"
        )
        .await,
        1
    );
}
