//! The card sync's upsert keeps going past a bad card: a batch that fails rolls back to its
//! savepoint, the card-by-card retry runs each card in its own, and every good card in the
//! batch lands while the bad one is recorded as an error.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "a test: helpers outside #[test] fns unwrap and index the fixture's JSON, and a panic is the failure report"
)]

use serde_json::Value;
use uuid::Uuid;
use zwipe::domain::card::models::zervice_metrics::ZerviceMetrics;
use zwipe::domain::card::ports::CardRepository;
use zwipe::outbound::sqlx::postgres::Postgres;
use zwipe_core::domain::card::scryfall_data::ScryfallData;

/// One real printing, the template the test's printings are made from.
const CARD: &str = include_str!("fixtures/share_preview_card.json");

/// Printing `n` of its own card, with `mtgo_id`, which the catalog holds unique.
fn printing(n: u128, mtgo_id: Option<i64>) -> ScryfallData {
    let mut card: Value = serde_json::from_str::<Value>(CARD).unwrap()["scryfall_data"].clone();
    card["id"] = Value::String(Uuid::from_u128(0x5a7e_0000 | n).to_string());
    card["oracle_id"] = Value::String(Uuid::from_u128(0x0a1c_0000 | n).to_string());
    card["name"] = Value::String(format!("Savepoint Card {n}"));
    card["collector_number"] = Value::String(n.to_string());
    card["mtgo_id"] = mtgo_id.map_or(Value::Null, Value::from);
    card["mtgo_foil_id"] = Value::Null;
    serde_json::from_value(card).unwrap()
}

/// The printings stored with ids in `0..count`.
async fn stored(pool: &sqlx::PgPool, count: u128) -> i64 {
    let ids: Vec<Uuid> = (0..count)
        .map(|n| Uuid::from_u128(0x5a7e_0000 | n))
        .collect();
    sqlx::query_scalar("SELECT count(*) FROM scryfall_data WHERE id = ANY($1)")
        .bind(ids)
        .fetch_one(pool)
        .await
        .unwrap()
}

#[sqlx::test]
async fn a_bad_card_in_a_batch_leaves_the_rest_of_the_night_upserted(pool: sqlx::PgPool) {
    let repo = Postgres::from_pool(pool.clone());
    let mut metrics = ZerviceMetrics::new();
    // Printing 0 holds mtgo id 777; printing 2 claims it too, so its batch fails.
    repo.batch_delta_upsert(&[printing(0, Some(777))], 100, &mut metrics)
        .await
        .unwrap();

    let mut metrics = ZerviceMetrics::new();
    let night = [
        printing(1, None),
        printing(2, Some(777)),
        printing(3, None),
        printing(4, None),
        printing(5, None),
    ];
    repo.batch_delta_upsert(&night, 2, &mut metrics)
        .await
        .unwrap();

    assert_eq!(
        stored(&pool, 6).await,
        5,
        "every printing but the bad one is stored"
    );
    assert_eq!(metrics.upserted_count(), 4);
    assert_eq!(metrics.error_count(), 1, "the bad printing is recorded");
    let errors = format!("{:?}", metrics.errors());
    assert!(errors.contains("Savepoint Card 2"), "{errors}");
    let holder = Uuid::from_u128(0x5a7e_0000);
    assert!(
        errors.contains(&format!(
            "mtgo_id 777 is already held by Savepoint Card 0 ({holder})"
        )),
        "the error names the clashing column and the card holding it: {errors}"
    );
}

#[sqlx::test]
async fn a_bad_card_in_a_plain_batch_upsert_leaves_the_rest_upserted(pool: sqlx::PgPool) {
    let repo = Postgres::from_pool(pool.clone());
    let mut metrics = ZerviceMetrics::new();
    repo.batch_upsert(&[printing(0, Some(888))], 100, &mut metrics)
        .await
        .unwrap();

    let mut metrics = ZerviceMetrics::new();
    let night = [printing(1, None), printing(2, Some(888)), printing(3, None)];
    repo.batch_upsert(&night, 3, &mut metrics).await.unwrap();

    assert_eq!(stored(&pool, 4).await, 3);
    assert_eq!(metrics.upserted_count(), 2);
    assert_eq!(metrics.error_count(), 1);
}
