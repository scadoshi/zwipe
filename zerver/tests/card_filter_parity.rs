//! SQL-vs-predicate parity: the backend SQL adapter (`search_cards`) and the
//! client-side in-memory filter (`CardCriteria::matches`) must accept/reject the
//! same cards for every criterion. This is the test that would have caught the
//! "field wired end-to-end except in the frontend predicate" class of bug (the
//! commander + legality bugs fixed in 1.0.2): a criterion that filters in SQL
//! but not in `matches` (or vice-versa) shows up here as a set mismatch.
//!
//! Method: seed one diverse universe, fetch it back as `Vec<Card>`, then for
//! each criterion assert `{cards SQL returns} == {cards matches() keeps}`.
//!
//! Excluded by design: `is_partner` / `is_background` / `is_signature_spell` —
//! `matches` deliberately does not evaluate these (command-zone pool constraints
//! the server applies only); see the doc comment on `CardCriteria::matches`.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

mod common;

use std::collections::BTreeSet;

use common::{card, seed_cards};
use serde_json::{Value, json};
use uuid::Uuid;

use zwipe::{domain::card::ports::CardRepository, outbound::sqlx::postgres::Postgres};
use zwipe_core::domain::card::{Card, search_card::card_filter::CardQuery};

/// Build a `CardQuery` from a flat criteria object, forcing a high limit so the
/// SQL side never truncates below the predicate side.
fn query(mut criteria: Value) -> CardQuery {
    criteria["limit"] = json!(250);
    serde_json::from_value(criteria).unwrap()
}

fn ids(cards: &[Card]) -> BTreeSet<Uuid> {
    cards.iter().map(|c| c.scryfall_data.id).collect()
}

#[sqlx::test]
async fn sql_and_predicate_agree_on_every_criterion(pool: sqlx::PgPool) {
    // A universe crafted for mixed match/exclude results across every filter.
    let universe = vec![
        card("Goblin Guide")
            .mono("R")
            .cmc(1.0)
            .type_line("Creature — Goblin")
            .power("2")
            .toughness("2")
            .keywords(&["Haste"])
            .rarity("rare")
            .set("M10", "Magic 2010")
            .artist("Alice Art")
            .usd("5.00")
            .flavor_text("Lightning fast")
            .categories(&["burn"])
            .legal("commander"),
        card("Llanowar Elves")
            .mono("G")
            .cmc(1.0)
            .type_line("Creature — Elf Druid")
            .power("1")
            .toughness("1")
            .produced_mana("G")
            .categories(&["ramp"])
            .rarity("common")
            .artist("Bob Art")
            .usd("0.50"),
        card("Counterspell")
            .mono("U")
            .cmc(2.0)
            .type_line("Instant")
            .oracle_text("Counter target spell.")
            .categories(&["counterspell"])
            .rarity("common"),
        card("Sol Ring")
            .color_identity("")
            .cmc(1.0)
            .type_line("Artifact")
            .categories(&["ramp"])
            .rarity("uncommon")
            .usd("2.00"),
        card("Wrath of God")
            .mono("W")
            .cmc(4.0)
            .type_line("Sorcery")
            .oracle_text("Destroy all creatures.")
            .categories(&["wipe", "removal"])
            .rarity("rare"),
        card("Lightning Bolt")
            .mono("R")
            .cmc(1.0)
            .type_line("Instant")
            .oracle_text("Lightning Bolt deals 3 damage to any target.")
            .categories(&["burn", "removal"])
            .rarity("common")
            .usd("1.00"),
        card("Krenko, Mob Boss")
            .mono("R")
            .cmc(4.0)
            .type_line("Legendary Creature — Goblin Warrior")
            .power("3")
            .toughness("3")
            .rarity("mythic")
            .legal("commander"),
        card("Island")
            .color_identity("")
            .cmc(0.0)
            .type_line("Basic Land — Island")
            .produced_mana("U")
            .rarity("common"),
        card("Forest")
            .color_identity("")
            .cmc(0.0)
            .type_line("Basic Land — Forest")
            .produced_mana("G")
            .rarity("common"),
        card("Goblin Token")
            .token()
            .mono("R")
            .cmc(0.0)
            .type_line("Token Creature — Goblin")
            .power("1")
            .toughness("1"),
        card("Digital Familiar")
            .mono("U")
            .cmc(3.0)
            .type_line("Creature — Spirit")
            .digital(true)
            .rarity("common"),
        card("Oversized Giant")
            .mono("W")
            .cmc(5.0)
            .type_line("Creature — Giant")
            .oversized(true),
        card("Promo Zombie")
            .mono("B")
            .cmc(2.0)
            .type_line("Creature — Zombie")
            .promo(true),
        card("Warning Horror")
            .mono("B")
            .cmc(3.0)
            .type_line("Creature — Horror")
            .content_warning(true),
        card("Ancestral Recall")
            .mono("U")
            .cmc(1.0)
            .type_line("Instant")
            .lang("ja")
            .rarity("common"),
        card("Black Lotus")
            .color_identity("")
            .cmc(0.0)
            .type_line("Artifact")
            .usd("10000.00")
            .rarity("rare")
            .flavor_text("Power nine"),
    ];
    let expected_universe = universe.len();
    seed_cards(&pool, &universe).await;

    let repo = Postgres { pool: pool.clone() };
    let all: Vec<Card> = repo.search_cards(&query(json!({}))).await.unwrap();
    assert_eq!(
        all.len(),
        expected_universe,
        "universe should round-trip whole"
    );

    // (label, criteria) — every branch of CardCriteria::matches except the three
    // documented server-only pool constraints.
    let battery: Vec<(&str, Value)> = vec![
        // text
        ("name_contains", json!({ "name_contains": "goblin" })),
        (
            "name_not_contains",
            json!({ "name_not_contains": "goblin" }),
        ),
        (
            "oracle_text_contains",
            json!({ "oracle_text_contains": "damage" }),
        ),
        (
            "oracle_text_not_contains",
            json!({ "oracle_text_not_contains": "counter" }),
        ),
        (
            "oracle_text_contains_any",
            json!({ "oracle_text_contains_any": ["destroy", "counter"] }),
        ),
        (
            "oracle_text_contains_all",
            json!({ "oracle_text_contains_all": ["target", "spell"] }),
        ),
        (
            "oracle_text_excludes_any",
            json!({ "oracle_text_excludes_any": ["damage"] }),
        ),
        // types
        (
            "type_line_contains",
            json!({ "type_line_contains": "instant" }),
        ),
        (
            "type_line_not_contains",
            json!({ "type_line_not_contains": "land" }),
        ),
        (
            "type_line_contains_any",
            json!({ "type_line_contains_any": ["instant", "sorcery"] }),
        ),
        (
            "type_line_contains_all",
            json!({ "type_line_contains_all": ["legendary", "creature"] }),
        ),
        (
            "type_line_excludes_any",
            json!({ "type_line_excludes_any": ["land"] }),
        ),
        (
            "card_type_contains_any",
            json!({ "card_type_contains_any": ["Instant"] }),
        ),
        (
            "card_type_contains_all",
            json!({ "card_type_contains_all": ["Creature"] }),
        ),
        (
            "card_type_excludes_any",
            json!({ "card_type_excludes_any": ["Land"] }),
        ),
        // keywords
        (
            "keywords_contains_any",
            json!({ "keywords_contains_any": ["haste"] }),
        ),
        (
            "keywords_contains_all",
            json!({ "keywords_contains_all": ["haste"] }),
        ),
        (
            "keywords_excludes",
            json!({ "keywords_excludes": ["haste"] }),
        ),
        // produced mana
        (
            "produced_mana_contains_any",
            json!({ "produced_mana_contains_any": ["G"] }),
        ),
        (
            "produced_mana_contains_all",
            json!({ "produced_mana_contains_all": ["G"] }),
        ),
        (
            "produced_mana_excludes",
            json!({ "produced_mana_excludes": ["G"] }),
        ),
        // mechanical categories
        (
            "mechanical_categories_contains_any",
            json!({ "mechanical_categories_contains_any": ["ramp"] }),
        ),
        (
            "mechanical_categories_contains_all",
            json!({ "mechanical_categories_contains_all": ["removal"] }),
        ),
        (
            "mechanical_categories_excludes",
            json!({ "mechanical_categories_excludes": ["burn"] }),
        ),
        // flavor
        (
            "flavor_text_contains",
            json!({ "flavor_text_contains": "fast" }),
        ),
        (
            "flavor_text_not_contains",
            json!({ "flavor_text_not_contains": "fast" }),
        ),
        ("has_flavor_text_true", json!({ "has_flavor_text": true })),
        ("has_flavor_text_false", json!({ "has_flavor_text": false })),
        // mana / combat
        ("cmc_equals", json!({ "cmc_equals": 1.0 })),
        ("cmc_range", json!({ "cmc_range": [2.0, 4.0] })),
        ("power_equals", json!({ "power_equals": 2 })),
        ("power_range", json!({ "power_range": [1, 3] })),
        ("toughness_equals", json!({ "toughness_equals": 2 })),
        ("toughness_range", json!({ "toughness_range": [1, 3] })),
        // price (usd is the default currency)
        ("price_max", json!({ "price_max": 5.0 })),
        ("price_min", json!({ "price_min": 100.0 })),
        // colors
        (
            "color_identity_equals",
            json!({ "color_identity_equals": ["R"] }),
        ),
        (
            "color_identity_within",
            json!({ "color_identity_within": ["R", "G"] }),
        ),
        // metadata
        (
            "rarity_equals_any",
            json!({ "rarity_equals_any": ["rare"] }),
        ),
        (
            "rarity_excludes_any",
            json!({ "rarity_excludes_any": ["common"] }),
        ),
        (
            "set_equals_any",
            json!({ "set_equals_any": ["Magic 2010"] }),
        ),
        (
            "set_excludes_any",
            json!({ "set_excludes_any": ["Test Set"] }),
        ),
        (
            "artist_equals_any",
            json!({ "artist_equals_any": ["Alice Art"] }),
        ),
        (
            "artist_excludes_any",
            json!({ "artist_excludes_any": ["Alice Art"] }),
        ),
        ("language", json!({ "language": "ja" })),
        // flags
        ("is_token_true", json!({ "is_token": true })),
        ("is_token_false", json!({ "is_token": false })),
        ("is_playable_true", json!({ "is_playable": true })),
        ("is_playable_false", json!({ "is_playable": false })),
        ("digital", json!({ "digital": true })),
        ("oversized", json!({ "oversized": true })),
        ("promo", json!({ "promo": true })),
        ("content_warning_true", json!({ "content_warning": true })),
        ("content_warning_false", json!({ "content_warning": false })),
        // legality + commander eligibility
        (
            "legalities_contains_any",
            json!({ "legalities_contains_any": ["commander"] }),
        ),
        (
            "is_commander_in_format",
            json!({ "is_commander_in_format": "commander" }),
        ),
    ];

    let mut mismatches: Vec<String> = Vec::new();
    for (label, criteria) in &battery {
        let q = query(criteria.clone());
        let sql = ids(&repo.search_cards(&q).await.unwrap());
        let predicate: BTreeSet<Uuid> = all
            .iter()
            .filter(|c| q.criteria().matches(c))
            .map(|c| c.scryfall_data.id)
            .collect();

        if sql != predicate {
            let names = |set: &BTreeSet<Uuid>| -> Vec<String> {
                all.iter()
                    .filter(|c| set.contains(&c.scryfall_data.id))
                    .map(|c| c.scryfall_data.name.clone())
                    .collect()
            };
            mismatches.push(format!(
                "`{label}`: SQL={:?} predicate={:?}",
                names(&sql),
                names(&predicate)
            ));
        }
    }

    assert!(
        mismatches.is_empty(),
        "SQL/predicate parity mismatches:\n{}",
        mismatches.join("\n")
    );
}
