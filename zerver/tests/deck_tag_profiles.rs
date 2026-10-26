//! The batched deck tag profile equals the single-deck one: seeds two decks'
//! worth of tagged cards, builds each profile both ways and compares them
//! tag by tag along with the batched card counts, with a deck whose cards carry no usable tags and a deck with
//! no cards both coming back `None`.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![allow(clippy::unwrap_used, clippy::indexing_slicing)]
#![expect(
    clippy::print_stderr,
    reason = "the test reports the elapsed time of each call"
)]

mod common;

use std::collections::HashMap;
use std::time::Instant;

use serde_json::Value;
use uuid::Uuid;
use zwipe::domain::card::ports::CardRepository;
use zwipe::outbound::sqlx::postgres::Postgres;

/// Writes the per-oracle tag rows both profile queries read.
async fn seed_oracle_tags(pool: &sqlx::PgPool, rows: &[(Uuid, &str)]) {
    for (oracle_id, tag) in rows {
        sqlx::query("INSERT INTO card_oracle_tags (oracle_id, oracle_tag) VALUES ($1, $2)")
            .bind(oracle_id)
            .bind(tag)
            .execute(pool)
            .await
            .unwrap();
    }
}

fn weights(profile: &Value) -> HashMap<String, f64> {
    profile
        .as_object()
        .unwrap()
        .iter()
        .map(|(tag, w)| (tag.clone(), w.as_f64().unwrap()))
        .collect()
}

fn assert_same_profile(single: &Value, batched: &Value, deck: &str) {
    let single = weights(single);
    let batched = weights(batched);
    let mut single_tags: Vec<&String> = single.keys().collect();
    let mut batched_tags: Vec<&String> = batched.keys().collect();
    single_tags.sort();
    batched_tags.sort();
    assert_eq!(single_tags, batched_tags, "{deck}: tag sets");
    for (tag, w) in &single {
        let b = batched[tag];
        assert!((w - b).abs() < 1e-9, "{deck}: {tag} single {w} batched {b}");
    }
    let sum: f64 = batched.values().sum();
    assert!((sum - 1.0).abs() < 1e-9, "{deck}: weights sum to {sum}");
}

#[sqlx::test]
async fn batched_profiles_match_the_single_deck_profile(pool: sqlx::PgPool) {
    let [a1, a2, a3, b1, b2, c1, x1, x2, x3] =
        [1_u128, 2, 3, 4, 5, 6, 7, 8, 9].map(Uuid::from_u128);
    seed_oracle_tags(
        &pool,
        &[
            (a1, "goblins"),
            (a2, "goblins"),
            (x1, "goblins"),
            (a1, "treasure"),
            (b1, "treasure"),
            (a3, "haste"),
            (b1, "lifegain"),
            (b2, "lifegain"),
            (x2, "lifegain"),
            (x3, "lifegain"),
            (a1, "cycle-foo"),
            (b1, "cycle-foo"),
            (c1, "cycle-bar"),
            (a1, "everywhere"),
            (a2, "everywhere"),
            (a3, "everywhere"),
            (b1, "everywhere"),
            (b2, "everywhere"),
            (c1, "everywhere"),
            (x1, "everywhere"),
            (x2, "everywhere"),
            (x3, "everywhere"),
        ],
    )
    .await;
    let repo = Postgres { pool: pool.clone() };

    let deck_a = vec![a1, a2, a3, a1];
    let deck_b = vec![b1, b2];
    let deck_c = vec![c1];
    let deck_d: Vec<Uuid> = vec![];

    let singles_started = Instant::now();
    let single_a = repo.deck_tag_profile(&deck_a, &[]).await.unwrap().unwrap();
    let single_b = repo.deck_tag_profile(&deck_b, &[]).await.unwrap().unwrap();
    let singles_elapsed = singles_started.elapsed();
    assert!(repo.deck_tag_profile(&deck_c, &[]).await.unwrap().is_none());

    let batched_started = Instant::now();
    let batched = repo
        .deck_tag_profiles(&[deck_a, deck_b, deck_c, deck_d])
        .await
        .unwrap();
    let batched_elapsed = batched_started.elapsed();
    eprintln!(
        "two singles {singles_elapsed:?}, one batched call for four decks {batched_elapsed:?}"
    );

    assert_eq!(batched.len(), 4);
    let batched_a = batched.first().unwrap().as_ref().unwrap();
    let batched_b = batched.get(1).unwrap().as_ref().unwrap();
    assert_same_profile(&single_a, &batched_a.weights, "deck a");
    assert_same_profile(&single_b, &batched_b.weights, "deck b");
    let counts = |cards: &[(&str, u32)]| -> HashMap<String, u32> {
        cards.iter().map(|(tag, n)| (tag.to_string(), *n)).collect()
    };
    assert_eq!(
        batched_a.cards,
        counts(&[("goblins", 2), ("treasure", 1), ("haste", 1)]),
        "deck a: distinct cards per kept tag, the repeated a1 counted once"
    );
    assert_eq!(
        batched_b.cards,
        counts(&[("treasure", 1), ("lifegain", 2)]),
        "deck b"
    );
    assert!(
        batched[2].is_none(),
        "only cycle and universal tags: {:?}",
        batched[2]
    );
    assert!(batched[3].is_none(), "no cards: {:?}", batched[3]);

    let a = weights(&single_a);
    assert!(a.contains_key("goblins") && a.contains_key("treasure") && a.contains_key("haste"));
    assert!(!a.contains_key("cycle-foo") && !a.contains_key("everywhere"));
    assert!(
        a["goblins"] > a["treasure"],
        "two goblins outweigh one treasure: {a:?}"
    );
}
