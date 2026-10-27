//! The batched deck tag profiles follow the lift steps in
//! `zwipe_core::domain::deck::tag_profile`: seeds a small corpus with tags
//! under a card role, a tag under none, a cycle tag, a tag on every card and a
//! tag the decks lean on less than the corpus does, then checks each deck's
//! weights against `profile_weights` run on the same counts, along with the
//! card counts, a deck whose cards carry no usable tags and a deck with no
//! cards both coming back `None`. The serve-time single profile keeps its own
//! weighting, so the test also pins where the two differ.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

mod common;

use std::collections::{BTreeMap, HashMap};

use serde_json::Value;
use uuid::Uuid;
use zwipe::domain::card::ports::CardRepository;
use zwipe::outbound::sqlx::postgres::Postgres;
use zwipe_core::domain::deck::tag_profile::{TagCounts, profile_weights};

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

/// Writes one catalog row, `parent` naming the parent tag's id.
async fn seed_catalog_tag(pool: &sqlx::PgPool, id: u128, slug: &str, parent: Option<u128>) {
    let parents: Vec<Uuid> = parent.map(Uuid::from_u128).into_iter().collect();
    sqlx::query("INSERT INTO oracle_tags (id, slug, label, parent_ids) VALUES ($1, $2, $2, $3)")
        .bind(Uuid::from_u128(id))
        .bind(slug)
        .bind(&parents)
        .execute(pool)
        .await
        .unwrap();
}

fn weights(profile: &Value) -> BTreeMap<String, f64> {
    profile
        .as_object()
        .unwrap()
        .iter()
        .map(|(tag, w)| (tag.clone(), w.as_f64().unwrap()))
        .collect()
}

fn assert_same_weights(got: &BTreeMap<String, f64>, want: &BTreeMap<String, f64>, deck: &str) {
    assert_eq!(
        got.keys().collect::<Vec<_>>(),
        want.keys().collect::<Vec<_>>(),
        "{deck}: tag sets"
    );
    for (tag, w) in want {
        let g = got[tag];
        assert!((w - g).abs() < 1e-9, "{deck}: {tag} sql {g} helper {w}");
    }
}

fn counts(slug: &str, deck_cards: u32, corpus_cards: u32) -> TagCounts {
    TagCounts {
        slug: slug.to_string(),
        deck_cards,
        corpus_cards,
    }
}

#[sqlx::test]
async fn batched_profiles_follow_the_lift_steps(pool: sqlx::PgPool) {
    let [a1, a2, a3, b1, b2, c1, x1, x2, x3, x4, x5, x6] =
        [1_u128, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12].map(Uuid::from_u128);
    let everyone = [a1, a2, a3, b1, b2, c1, x1, x2, x3, x4, x5, x6];
    let mut rows: Vec<(Uuid, &str)> = everyone.iter().map(|id| (*id, "removal")).collect();
    rows.extend([a1, a2, x1, x2, x3, x4].map(|id| (id, "ramp")));
    rows.push((a3, "tutor"));
    rows.extend([a1, b1, x1, x2, x3, x4, x5, x6].map(|id| (id, "lifegain")));
    rows.extend([b2, x1].map(|id| (id, "burn")));
    rows.push((b1, "sacrifice-outlet"));
    rows.extend([a1, a2].map(|id| (id, "typal-goblin")));
    rows.extend([a1, b1].map(|id| (id, "cycle-ramp")));
    rows.push((c1, "cycle-bar"));
    seed_oracle_tags(&pool, &rows).await;
    // Role roots in the catalog; cycle-ramp sits under ramp, so only its
    // prefix keeps it out. typal-goblin has no row and falls under no role.
    for (id, slug) in [
        (0x10, "removal"),
        (0x11, "ramp"),
        (0x12, "tutor"),
        (0x13, "lifegain"),
        (0x14, "burn"),
        (0x15, "sacrifice-outlet"),
    ] {
        seed_catalog_tag(&pool, id, slug, None).await;
    }
    seed_catalog_tag(&pool, 0x16, "cycle-ramp", Some(0x11)).await;
    let repo = Postgres { pool: pool.clone() };

    let deck_a = vec![a1, a2, a3, a1];
    let deck_b = vec![b1, b2];
    let deck_c = vec![c1];
    let deck_d: Vec<Uuid> = vec![];
    let batched = repo
        .deck_tag_profiles(&[deck_a.clone(), deck_b, deck_c, deck_d])
        .await
        .unwrap();
    assert_eq!(batched.len(), 4);

    // Twelve tagged cards in the corpus. Every tag in the counts below is
    // under a role except typal-goblin.
    let has_role = |slug: &str| slug != "typal-goblin";
    let profile_a = batched[0].as_ref().unwrap();
    let want_a = profile_weights(
        &[
            counts("removal", 3, 12),
            counts("ramp", 2, 6),
            counts("tutor", 1, 1),
            counts("lifegain", 1, 8),
            counts("typal-goblin", 2, 2),
            counts("cycle-ramp", 1, 2),
        ],
        3,
        12,
        has_role,
    );
    let got_a = weights(&profile_a.weights);
    assert_same_weights(&got_a, &want_a, "deck a");
    assert!(
        got_a["tutor"] > got_a["ramp"],
        "one tutor at lift 4 beats two ramp cards at lift 4/3: {got_a:?}"
    );
    let card_counts = |cards: &[(&str, u32)]| -> HashMap<String, u32> {
        cards.iter().map(|(tag, n)| (tag.to_string(), *n)).collect()
    };
    assert_eq!(
        profile_a.cards,
        card_counts(&[("ramp", 2), ("tutor", 1)]),
        "deck a: distinct cards per kept tag, the repeated a1 counted once"
    );

    let profile_b = batched[1].as_ref().unwrap();
    let want_b = profile_weights(
        &[
            counts("removal", 2, 12),
            counts("lifegain", 1, 8),
            counts("burn", 1, 2),
            counts("sacrifice-outlet", 1, 1),
            counts("cycle-ramp", 1, 2),
        ],
        2,
        12,
        has_role,
    );
    assert_same_weights(&weights(&profile_b.weights), &want_b, "deck b");
    assert_eq!(
        profile_b.cards,
        card_counts(&[("burn", 1), ("sacrifice-outlet", 1)]),
        "deck b"
    );
    for profile in [profile_a, profile_b] {
        let sum: f64 = weights(&profile.weights).values().sum();
        assert!((sum - 1.0).abs() < 1e-9, "weights sum to {sum}");
    }
    assert!(
        batched[2].is_none(),
        "only a cycle tag and a tag on every card: {:?}",
        batched[2]
    );
    assert!(batched[3].is_none(), "no cards: {:?}", batched[3]);

    // The serve-time profile still weighs by rarity alone, so the generic
    // lifegain and the role-less typal-goblin stay in it.
    let single_a = weights(&repo.deck_tag_profile(&deck_a, &[]).await.unwrap().unwrap());
    assert!(
        single_a.contains_key("lifegain") && single_a.contains_key("typal-goblin"),
        "{single_a:?}"
    );
    assert!(!got_a.contains_key("lifegain") && !got_a.contains_key("typal-goblin"));
}
