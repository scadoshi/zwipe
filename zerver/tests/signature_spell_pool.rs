//! The Oathbreaker signature-spell pool, as the SQL adapter serves it.
//!
//! `CardCriteria::matches` deliberately does not evaluate `is_signature_spell`
//! (it is a server-only pool constraint), so `card_filter_parity` excludes it
//! and nothing else covered this filter. It needs its own test.
//!
//! The rule: a signature spell is an instant or sorcery CARD. An Adventure
//! creature is a creature card whose Adventure half only becomes a spell on
//! the stack, and the Oathbreaker FAQ rejects it by name. Split cards are the
//! exception, being both halves at once outside the stack (CR 709.4).
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

mod common;

use std::collections::BTreeSet;

use common::{card, seed_cards};
use serde_json::json;

use zwipe::{domain::card::ports::CardRepository, outbound::sqlx::postgres::Postgres};
use zwipe_core::domain::card::{Card, search_card::card_filter::CardQuery};

fn names(cards: &[Card]) -> BTreeSet<String> {
    cards.iter().map(|c| c.scryfall_data.name.clone()).collect()
}

#[sqlx::test]
async fn signature_spell_pool_reads_the_front_face(pool: sqlx::PgPool) {
    let universe = vec![
        // Eligible: plain spells.
        card("Lightning Bolt").mono("R").type_line("Instant"),
        card("Ponder").mono("U").type_line("Sorcery"),
        // Eligible: the spell is the front face, the land is the back.
        card("Hagra Mauling // Hagra Broodpit")
            .mono("B")
            .layout("modal_dfc")
            .type_line("Instant // Land"),
        // Eligible: a split card is both halves at once.
        card("Fire // Ice")
            .color_identity("UR")
            .layout("split")
            .type_line("Instant // Instant"),
        // Rejected: creature cards carrying a spell half.
        card("Beluna Grandsquall // Seek Thrills")
            .mono("G")
            .layout("adventure")
            .type_line("Legendary Creature — Giant Noble // Instant — Adventure")
            .power("3")
            .toughness("3"),
        card("Adventurous Eater // Have a Bite")
            .mono("B")
            .layout("prepare")
            .type_line("Creature — Human Warlock // Sorcery")
            .power("2")
            .toughness("2"),
        // Rejected: a land that goes on an adventure is still a land card.
        card("Midgar, City of Mako // Reactor Raid")
            .color_identity("R")
            .layout("adventure")
            .type_line("Land — Town // Sorcery — Adventure"),
        // Rejected: split layout, but neither half is a spell.
        card("Cramped Vents // Access Maze")
            .mono("R")
            .layout("split")
            .type_line("Enchantment — Room // Enchantment — Room"),
        // Rejected: not a spell at all.
        card("Llanowar Elves")
            .mono("G")
            .type_line("Creature — Elf Druid")
            .power("1")
            .toughness("1"),
    ];
    seed_cards(&pool, &universe).await;

    let repo = Postgres { pool: pool.clone() };
    let criteria = json!({ "is_signature_spell": true, "limit": 250 });
    let found: Vec<Card> = repo
        .search_cards(&serde_json::from_value::<CardQuery>(criteria).unwrap())
        .await
        .unwrap();

    let expected: BTreeSet<String> = [
        "Lightning Bolt",
        "Ponder",
        "Hagra Mauling // Hagra Broodpit",
        "Fire // Ice",
    ]
    .iter()
    .map(|s| (*s).to_string())
    .collect();

    assert_eq!(
        names(&found),
        expected,
        "signature-spell pool should hold only instant and sorcery cards"
    );
}
