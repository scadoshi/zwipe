//! Repo-level card tests: the SQL whose logic doesn't surface cleanly through
//! HTTP. Constructs `Postgres { pool }` and calls `CardRepository` methods
//! directly (no router). Covers the default synergy ordering, the
//! `card_signal_rollup` math, and the deck-aware serve's NULL-`oracle_id`
//! handling (the 2026-07-06 regression: `NULL || seed` NULLed the shuffle key).
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

mod common;

use common::{card, refresh_card_views, seed_cards};
use serde_json::json;
use uuid::Uuid;

use zwipe::{
    domain::{
        card::ports::{CardRepository, DeckServeContext},
        deck::ports::DeckRepository,
    },
    outbound::sqlx::postgres::Postgres,
};
use zwipe_core::domain::{
    card::{
        scryfall_data::universe::OUT_OF_UNIVERSE_SETS,
        search_card::card_filter::{CardQuery, builder::CardQueryBuilder},
    },
    deck::requests::get_deck_profile::GetDeckProfile,
};

/// A default `CardQuery` — no criteria, no explicit sort (so the synergy /
/// popularity ordering is the one under test).
fn default_query() -> CardQuery {
    serde_json::from_value(json!({})).unwrap()
}

async fn insert_signal(
    pool: &sqlx::PgPool,
    commander: Uuid,
    card_oracle: Uuid,
    shown: i64,
    added: i64,
    maybed: i64,
    removed: i64,
) {
    sqlx::query(
        "INSERT INTO commander_card_signal \
         (commander_oracle_id, card_oracle_id, shown, added, maybed, removed) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(commander)
    .bind(card_oracle)
    .bind(shown)
    .bind(added)
    .bind(maybed)
    .bind(removed)
    .execute(pool)
    .await
    .unwrap();
}

/// The rollup collapses every commander's rows for a card into one net/shown
/// pair: `net = Σ(added + 0.5·maybed − removed)`, `shown = Σ shown`.
#[sqlx::test]
async fn card_signal_rollup_math(pool: sqlx::PgPool) {
    let card_oracle = Uuid::from_u128(0xCA2D);
    let c1 = Uuid::from_u128(0xC001);
    let c2 = Uuid::from_u128(0xC002);

    // c1: 4 + 0.5*2 - 1 = 4.0 over 10 shown; c2: 1 + 0 - 0 = 1.0 over 5 shown.
    insert_signal(&pool, c1, card_oracle, 10, 4, 2, 1).await;
    insert_signal(&pool, c2, card_oracle, 5, 1, 0, 0).await;
    // a different card, to prove the GROUP BY isolates rows
    insert_signal(&pool, c1, Uuid::from_u128(0xBEEF), 3, 3, 0, 0).await;

    refresh_card_views(&pool).await;

    let (net, shown): (f64, f64) =
        sqlx::query_as("SELECT net, shown FROM card_signal_rollup WHERE card_oracle_id = $1")
            .bind(card_oracle)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(net, 5.0, "net = (4 + 1 - 1) + (1)");
    assert_eq!(shown, 15.0, "shown = 10 + 5");

    // the isolated card carries only its own row
    let (net2, shown2): (f64, f64) =
        sqlx::query_as("SELECT net, shown FROM card_signal_rollup WHERE card_oracle_id = $1")
            .bind(Uuid::from_u128(0xBEEF))
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!((net2, shown2), (3.0, 3.0));
}

/// With no deck seed, the deck-aware serve is pure score order: scored cards by
/// synergy score descending, unscored cards anchored below the floor
/// (UNSCORED_ANCHOR). No banding, no shuffle — deterministic.
#[sqlx::test]
async fn synergy_ordering_scored_before_unscored(pool: sqlx::PgPool) {
    seed_cards(
        &pool,
        &[
            card("Alpha Card").mono("R"),
            card("Bravo Card").mono("R"),
            card("Zeta Card").mono("R"), // unscored
        ],
    )
    .await;

    // keys are LOWER(name); Zeta is absent so it takes the anchor.
    let scores = json!({ "alpha card": 5.0, "bravo card": 1.0 });
    let repo = Postgres { pool: pool.clone() };

    let served = repo
        .search_scryfall_data_deck_aware(
            &default_query(),
            DeckServeContext {
                synergy_scores: Some(&scores),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let names: Vec<&str> = served.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, vec!["Alpha Card", "Bravo Card", "Zeta Card"]);
}

/// `exclude_oracle_ids` drops a card from the deck-aware serve (the deck's own
/// cards must not be re-served).
#[sqlx::test]
async fn deck_aware_serve_excludes_oracle_ids(pool: sqlx::PgPool) {
    let keep = card("Keeper").mono("R");
    let drop = card("Dropped").mono("R");
    let drop_oracle = drop.oracle_id().unwrap();
    seed_cards(&pool, &[keep, drop]).await;

    let scores = json!({});
    let repo = Postgres { pool: pool.clone() };
    let served = repo
        .search_scryfall_data_deck_aware(
            &default_query(),
            DeckServeContext {
                exclude_oracle_ids: &[drop_oracle],
                synergy_scores: Some(&scores),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let names: Vec<&str> = served.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names,
        vec!["Keeper"],
        "excluded oracle_id must not be served"
    );
}

/// Regression (2026-07-06): a card with a NULL `oracle_id` must survive the
/// deck-seeded banded shuffle. The shuffle key is
/// `hashtext(COALESCE(oracle_id::text, '') || seed)`; before the COALESCE,
/// `NULL || seed` NULLed the key and mis-ordered/hid these cards. Here we assert
/// the NULL-oracle card is still served, the whole set comes back, and the
/// order is deterministic for a fixed deck seed.
#[sqlx::test]
async fn null_oracle_card_survives_deck_aware_shuffle(pool: sqlx::PgPool) {
    seed_cards(
        &pool,
        &[
            card("Real One").mono("R"),
            card("Null Oracle").mono("R").oracle(None),
            card("Real Two").mono("R"),
            card("Real Three").mono("R"),
        ],
    )
    .await;

    let scores = json!({});
    let deck = Uuid::from_u128(0xDEC0);
    let repo = Postgres { pool: pool.clone() };
    let q = default_query();

    let first = repo
        .search_scryfall_data_deck_aware(
            &q,
            DeckServeContext {
                deck_id: Some(deck),
                synergy_scores: Some(&scores),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let names: Vec<&str> = first.iter().map(|s| s.name.as_str()).collect();

    assert_eq!(
        first.len(),
        4,
        "all four cards served, none dropped: {names:?}"
    );
    assert!(
        names.contains(&"Null Oracle"),
        "NULL-oracle card must be served: {names:?}"
    );

    // deterministic for a fixed (deck, day) seed
    let second = repo
        .search_scryfall_data_deck_aware(
            &q,
            DeckServeContext {
                deck_id: Some(deck),
                synergy_scores: Some(&scores),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let names2: Vec<&str> = second.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, names2, "same deck seed must yield the same order");
}

/// Phase 4: a deck's selected oracle tags lift matching cards within the
/// synergy serve. Both cards are unscored (equal base), so the otag term is the
/// only differentiator — and with no selected otags the term is dormant and the
/// name tiebreak wins (the zero-regression / revert guarantee).
#[sqlx::test]
async fn deck_oracle_tags_lift_matching_cards(pool: sqlx::PgPool) {
    seed_cards(
        &pool,
        &[
            // Alphabetically first, so it leads on the name tiebreak absent otags.
            card("Aaa Plain").mono("R"),
            card("Zzz Removal").mono("R").oracle_tags(&["spot-removal"]),
        ],
    )
    .await;

    // Empty score map: both cards take the unscored anchor, so base + signal are
    // equal and the otag term alone decides the order.
    let scores = json!({});
    let repo = Postgres { pool: pool.clone() };

    // With the deck's selected otag, the matching card is lifted to the front,
    // overriding the alphabetical tiebreak.
    let served = repo
        .search_scryfall_data_deck_aware(
            &default_query(),
            DeckServeContext {
                synergy_scores: Some(&scores),
                deck_oracle_tags: &["spot-removal".to_string()],
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let names: Vec<&str> = served.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names,
        vec!["Zzz Removal", "Aaa Plain"],
        "otag-matching card should lead when the deck selected that otag"
    );

    // Zero-regression: no selected otags => the term is dormant and ordering
    // falls back to the name tiebreak (byte-identical to pre-Phase-4).
    let served = repo
        .search_scryfall_data_deck_aware(
            &default_query(),
            DeckServeContext {
                synergy_scores: Some(&scores),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let names: Vec<&str> = served.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names,
        vec!["Aaa Plain", "Zzz Removal"],
        "no selected otags => name order, no otag lift"
    );
}

/// Phase 4 (production path): the otag term must reorder within the *banded /
/// wildcard* serve (deck_id present), not just the pure-score path. 30 unscored
/// cards; four otag-matching cards are named to sort last, so absent otags they
/// land in band 1 (off the 24-card first page). With the deck's selected otag
/// the `W_ORACLE_TAG` lift pulls them into band 0 and onto the first page. This
/// exercises the wildcard CTE's `push_score` ranking (the path real serves use).
#[sqlx::test]
async fn deck_oracle_tags_lift_matching_cards_in_banded_serve(pool: sqlx::PgPool) {
    let mut fixtures: Vec<_> = (1..=26)
        .map(|i| card(&format!("Aaa{i:02}")).mono("R"))
        .collect();
    for i in 1..=4 {
        fixtures.push(
            card(&format!("Zzz{i:02}"))
                .mono("R")
                .oracle_tags(&["spot-removal"]),
        );
    }
    seed_cards(&pool, &fixtures).await;

    // Empty score map => every card takes the unscored anchor, so the otag term
    // is the only thing that moves a card between bands.
    let scores = json!({});
    let repo = Postgres { pool: pool.clone() };
    let deck = Uuid::from_u128(0xB4_0004);
    let q = default_query();

    let served_without = repo
        .search_scryfall_data_deck_aware(
            &q,
            DeckServeContext {
                deck_id: Some(deck),
                synergy_scores: Some(&scores),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let zzz_without = served_without
        .iter()
        .filter(|s| s.name.starts_with("Zzz"))
        .count();
    assert_eq!(
        zzz_without, 0,
        "absent otags, the otag cards sit in band 1, off the first page"
    );

    let served_with = repo
        .search_scryfall_data_deck_aware(
            &q,
            DeckServeContext {
                deck_id: Some(deck),
                synergy_scores: Some(&scores),
                deck_oracle_tags: &["spot-removal".to_string()],
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let zzz_with = served_with
        .iter()
        .filter(|s| s.name.starts_with("Zzz"))
        .count();
    assert!(
        zzz_with >= 1,
        "with the deck's selected otag, matching cards are lifted onto the first page (got {zzz_with})"
    );
}

/// MVP steering (deck-MVPs phase 3) reads only the roles that should steer:
/// mainboard stars, deduped, with unstarred cards ignored. The serve term is a
/// flat overlap lift, so a role shared by two MVPs must appear once — a tally
/// here would silently double that role's weight. A non-mainboard star can't be
/// constructed at all now (`deck_cards_mvp_mainboard_only`); the import path
/// that used to strand one is covered in `import_atomicity.rs`.
#[sqlx::test]
async fn mvp_card_roles_are_mainboard_stars_deduped(pool: sqlx::PgPool) {
    let ramp = card("Ramp One").categories(&["ramp", "draw"]);
    let ramp2 = card("Ramp Two").categories(&["ramp"]);
    let side = card("Side Card").categories(&["wipe"]);
    let plain = card("Not Starred").categories(&["removal"]);
    // (scryfall id, oracle id, board, starred)
    let rows = [
        (ramp.id(), ramp.oracle_id().unwrap(), "deck", true),
        (ramp2.id(), ramp2.oracle_id().unwrap(), "deck", true),
        (side.id(), side.oracle_id().unwrap(), "maybeboard", false),
        (plain.id(), plain.oracle_id().unwrap(), "deck", false),
    ];
    seed_cards(&pool, &[ramp, ramp2, side, plain]).await;

    let user_id: Uuid = sqlx::query_scalar(
        "INSERT INTO users (username, email, password_hash) VALUES ('mvpsteer', 'mvpsteer@x.co', 'x') RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let deck_id: Uuid = sqlx::query_scalar(
        "INSERT INTO decks (name, format, user_id) VALUES ('Steer', 'commander', $1) RETURNING id",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    for (sd_id, oracle_id, board, starred) in rows {
        sqlx::query(
            "INSERT INTO deck_cards (deck_id, scryfall_data_id, oracle_id, quantity, board, mvp_at)
             VALUES ($1, $2, $3, 1, $4, CASE WHEN $5 THEN now() ELSE NULL END)",
        )
        .bind(deck_id)
        .bind(sd_id)
        .bind(oracle_id)
        .bind(board)
        .bind(starred)
        .execute(&pool)
        .await
        .unwrap();
    }

    let repo = Postgres { pool: pool.clone() };
    let request = GetDeckProfile { user_id, deck_id };
    let mut roles = repo.get_mvp_card_roles(&request).await.unwrap();
    roles.sort();

    // "ramp" is on both mainboard MVPs but must appear once; "wipe" sits on an
    // unstarred maybeboard card and "removal" on an unstarred mainboard card,
    // so neither steers.
    assert_eq!(roles, vec!["draw".to_string(), "ramp".to_string()]);
}

/// The `W_STEER` term lifts cards sharing a role with the deck's MVPs, and is
/// dormant when the deck has none. Mirrors the otag-lift test: an empty score
/// map puts both cards on the unscored anchor, so the steering term alone
/// decides the order and the alphabetical tiebreak shows when it is off.
#[sqlx::test]
async fn mvp_roles_lift_matching_cards(pool: sqlx::PgPool) {
    seed_cards(
        &pool,
        &[
            // Alphabetically first, so it leads absent any steering.
            card("Aaa Plain").mono("R"),
            card("Zzz Ramp").mono("R").categories(&["ramp"]),
        ],
    )
    .await;

    let scores = json!({});
    let repo = Postgres { pool: pool.clone() };

    // Deck whose MVPs are ramp: the ramp card overtakes the alphabetical lead.
    let served = repo
        .search_scryfall_data_deck_aware(
            &default_query(),
            DeckServeContext {
                synergy_scores: Some(&scores),
                mvp_card_roles: &["ramp".to_string()],
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let names: Vec<&str> = served.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names,
        vec!["Zzz Ramp", "Aaa Plain"],
        "a card sharing an MVP role should lead"
    );

    // No MVPs: the term is dormant and the plain alphabetical order returns.
    let served = repo
        .search_scryfall_data_deck_aware(
            &default_query(),
            DeckServeContext {
                synergy_scores: Some(&scores),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let names: Vec<&str> = served.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names,
        vec!["Aaa Plain", "Zzz Ramp"],
        "a deck with no MVPs must order exactly as before"
    );
}

/// The oou_sets overlay inside `refresh_latest_cards` must make the table
/// exactly mirror zwipe-core's OUT_OF_UNIVERSE_SETS const: additions land,
/// removals prune, and the migration seed drifting from the const self-heals
/// on the next run.
#[sqlx::test]
async fn oou_sets_overlay_syncs_table_to_const(pool: sqlx::PgPool) {
    let repo = Postgres { pool: pool.clone() };

    // Drift the seeded table both ways: a code the const doesn't carry, and a
    // real code gone missing.
    sqlx::query("INSERT INTO oou_sets (code) VALUES ('zzz')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM oou_sets WHERE code = 'spm'")
        .execute(&pool)
        .await
        .unwrap();

    repo.refresh_latest_cards().await.unwrap();

    let in_db: Vec<String> = sqlx::query_scalar("SELECT code FROM oou_sets ORDER BY code")
        .fetch_all(&pool)
        .await
        .unwrap();
    let mut expected: Vec<String> = OUT_OF_UNIVERSE_SETS
        .iter()
        .map(|code| (*code).to_string())
        .collect();
    expected.sort();
    assert_eq!(in_db, expected, "oou_sets must mirror the const exactly");
}

/// The in-universe ORDER BY tiebreaker: with two same-day printings of one
/// card, the pick must be the in-universe one, and an OOU-only card keeps its
/// OOU pick (all rows tie on the term). Also proves printing_set_names
/// aggregates both printings.
#[sqlx::test]
async fn latest_cards_pick_prefers_in_universe_printing(pool: sqlx::PgPool) {
    let oracle = Uuid::from_u128(0x0001);
    seed_cards(
        &pool,
        &[
            card("Universal Staple")
                .mono("R")
                .oracle(Some(oracle))
                .set("tst", "Test Set"),
            card("Universal Staple")
                .mono("R")
                .oracle(Some(oracle))
                .set("spm", "Marvel's Spider-Man"),
            card("Beyond Only")
                .mono("R")
                .set("spm", "Marvel's Spider-Man"),
        ],
    )
    .await;

    let (set_name, all_sets): (String, Vec<String>) = sqlx::query_as(
        "SELECT set_name, printing_set_names FROM latest_cards WHERE name = 'Universal Staple'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        set_name, "Test Set",
        "in-universe printing must win the pick"
    );
    let mut all_sets = all_sets;
    all_sets.sort();
    assert_eq!(all_sets, vec!["Marvel's Spider-Man", "Test Set"]);

    let beyond_pick: String =
        sqlx::query_scalar("SELECT set_name FROM latest_cards WHERE name = 'Beyond Only'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(
        beyond_pick, "Marvel's Spider-Man",
        "an OOU-only card keeps its OOU pick"
    );
}

/// Array-semantics set filters: include matches any printing's set (not just
/// the pick), and exclude only hides a card when EVERY printing's set is
/// excluded — the Sol Ring / Secret Lair Drop shadowing bug.
#[sqlx::test]
async fn set_filters_are_printing_aware(pool: sqlx::PgPool) {
    let oracle = Uuid::from_u128(0x0002);
    seed_cards(
        &pool,
        &[
            card("Universal Staple")
                .mono("R")
                .oracle(Some(oracle))
                .set("tst", "Test Set"),
            card("Universal Staple")
                .mono("R")
                .oracle(Some(oracle))
                .set("spm", "Marvel's Spider-Man"),
            card("Beyond Only")
                .mono("R")
                .set("spm", "Marvel's Spider-Man"),
        ],
    )
    .await;
    let repo = Postgres { pool: pool.clone() };

    // Include: the staple's pick is Test Set, but its spm printing must match.
    let mut b = CardQueryBuilder::new();
    b.set_set_equals_any(vec!["Marvel's Spider-Man"]);
    let served = repo
        .search_scryfall_data(&b.build().unwrap())
        .await
        .unwrap();
    let mut names: Vec<&str> = served.iter().map(|s| s.name.as_str()).collect();
    names.sort();
    assert_eq!(
        names,
        vec!["Beyond Only", "Universal Staple"],
        "include must match any printing's set"
    );

    // Exclude: the staple survives (it also exists in Test Set); the
    // spm-only card is gone.
    let mut b = CardQueryBuilder::new();
    b.set_set_excludes_any(vec!["Marvel's Spider-Man"]);
    let served = repo
        .search_scryfall_data(&b.build().unwrap())
        .await
        .unwrap();
    let names: Vec<&str> = served.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names,
        vec!["Universal Staple"],
        "exclude must only hide cards with no printing left"
    );
}
