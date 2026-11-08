//! An Archidekt import fills the deck's empty format and commander slots,
//! keeps the commanders off the board it imports into, and leaves filled
//! slots alone.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "a test: helpers unwrap and index, and a panic is the failure report"
)]

mod common;

use axum::http::StatusCode;
use common::{CardFixture, TestApp, card, seed_cards};
use serde_json::{Value, json};
use uuid::Uuid;
use zwipe::domain::deck::models::deck::import_archidekt::{ArchidektCard, ArchidektDeck};
use zwipe::domain::deck::ports::DeckService;
use zwipe::domain::deck::services::Service;
use zwipe::outbound::sqlx::postgres::Postgres;
use zwipe_core::domain::deck::requests::import_deck_cards::ImportDeckCardsResult;
use zwipe_core::domain::deck::{Board, Format, ImportMode};
use zwipe_core::http::paths::DECK_ROUTE;

/// A verified user and a deck made with `body`, as `(user_id, deck_id)`.
async fn deck_for(app: &TestApp, username: &str, body: Value) -> (Uuid, Uuid) {
    let (token, uid) = app.register(username).await;
    app.verify_email(&uid).await;
    let (status, deck) = app.post(DECK_ROUTE, body, Some(&token)).await;
    assert_eq!(status, StatusCode::CREATED, "deck create: {deck}");
    (
        uid.parse().unwrap(),
        deck["id"].as_str().unwrap().parse().unwrap(),
    )
}

fn entry(fixture: &CardFixture, name: &str) -> ArchidektCard {
    ArchidektCard {
        scryfall_id: fixture.id(),
        name: name.to_owned(),
        quantity: 1,
    }
}

async fn import(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    deck_id: Uuid,
    deck: &ArchidektDeck,
) -> ImportDeckCardsResult {
    let db = Postgres::from_pool(pool.clone());
    Service::new(db.clone(), db)
        .import_archidekt_deck(user_id, deck_id, deck, Board::Deck, true, ImportMode::Add)
        .await
        .unwrap()
}

/// The deck's format and command zone ids, as stored.
async fn profile(pool: &sqlx::PgPool, deck_id: Uuid) -> (Option<String>, [Option<Uuid>; 3]) {
    let (format, commander, partner, background): (
        Option<String>,
        Option<Uuid>,
        Option<Uuid>,
        Option<Uuid>,
    ) = sqlx::query_as(
        "SELECT format, commander_id, partner_commander_id, background_id FROM decks WHERE id = $1",
    )
    .bind(deck_id)
    .fetch_one(pool)
    .await
    .unwrap();
    (format, [commander, partner, background])
}

async fn board_ids(pool: &sqlx::PgPool, deck_id: Uuid) -> Vec<Uuid> {
    sqlx::query_scalar("SELECT scryfall_data_id FROM deck_cards WHERE deck_id = $1")
        .bind(deck_id)
        .fetch_all(pool)
        .await
        .unwrap()
}

#[sqlx::test]
async fn an_empty_deck_takes_the_format_and_both_partners(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (user_id, deck_id) = deck_for(&app, "fillpartners", json!({ "name": "Blank" })).await;
    let tymna = card("Tymna the Weaver").type_line("Legendary Creature — Human Cleric");
    let thrasios = card("Thrasios, Triton Hero").type_line("Legendary Creature — Merfolk Wizard");
    let remora = card("Mystic Remora").type_line("Enchantment");
    seed_cards(&pool, &[tymna.clone(), thrasios.clone(), remora.clone()]).await;

    let commanders = vec![
        entry(&tymna, "Tymna the Weaver"),
        entry(&thrasios, "Thrasios, Triton Hero"),
    ];
    let mut cards = commanders.clone();
    cards.push(entry(&remora, "Mystic Remora"));
    let result = import(
        &pool,
        user_id,
        deck_id,
        &ArchidektDeck {
            cards,
            format: Some(Format::Commander),
            commanders,
        },
    )
    .await;

    assert_eq!(result.format_set, Some(Format::Commander));
    assert_eq!(
        result.commanders_set,
        ["Tymna the Weaver", "Thrasios, Triton Hero"]
    );
    let (format, zone) = profile(&pool, deck_id).await;
    assert_eq!(format.as_deref(), Some("commander"));
    assert_eq!(zone, [Some(tymna.id()), Some(thrasios.id()), None]);
    assert_eq!(
        board_ids(&pool, deck_id).await,
        [remora.id()],
        "the commanders stay off the board"
    );
}

#[sqlx::test]
async fn a_background_takes_the_background_slot(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (user_id, deck_id) = deck_for(&app, "fillbackground", json!({ "name": "Blank" })).await;
    let giants = card("Raised by Giants").type_line("Legendary Enchantment — Background");
    let wyll = card("Wyll, Blade of Frontiers").type_line("Legendary Creature — Human Warlock");
    seed_cards(&pool, &[giants.clone(), wyll.clone()]).await;

    let commanders = vec![
        entry(&giants, "Raised by Giants"),
        entry(&wyll, "Wyll, Blade of Frontiers"),
    ];
    let result = import(
        &pool,
        user_id,
        deck_id,
        &ArchidektDeck {
            cards: commanders.clone(),
            format: Some(Format::Commander),
            commanders,
        },
    )
    .await;

    assert_eq!(
        result.commanders_set,
        ["Wyll, Blade of Frontiers", "Raised by Giants"]
    );
    let (_, zone) = profile(&pool, deck_id).await;
    assert_eq!(zone, [Some(wyll.id()), None, Some(giants.id())]);
}

#[sqlx::test]
async fn filled_slots_stay_and_the_result_says_nothing_was_set(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let kenrith = card("Kenrith, the Returned King").type_line("Legendary Creature — Human Noble");
    let atraxa = card("Atraxa, Praetors' Voice").type_line("Legendary Creature — Phyrexian Angel");
    seed_cards(&pool, &[kenrith.clone(), atraxa.clone()]).await;
    let (user_id, deck_id) = deck_for(
        &app,
        "fillkept",
        json!({ "name": "Kept", "format": "duel", "commander_id": kenrith.id() }),
    )
    .await;

    let commanders = vec![entry(&atraxa, "Atraxa, Praetors' Voice")];
    let result = import(
        &pool,
        user_id,
        deck_id,
        &ArchidektDeck {
            cards: commanders.clone(),
            format: Some(Format::Commander),
            commanders,
        },
    )
    .await;

    assert_eq!(result.format_set, None);
    assert_eq!(result.commanders_set, Vec::<String>::new());
    let (format, zone) = profile(&pool, deck_id).await;
    assert_eq!(format.as_deref(), Some("duel"));
    assert_eq!(zone, [Some(kenrith.id()), None, None]);
    assert_eq!(
        board_ids(&pool, deck_id).await,
        [atraxa.id()],
        "a commander the deck didn't take imports as a card"
    );
}

#[sqlx::test]
async fn an_unmapped_format_sets_the_commander_only(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (user_id, deck_id) = deck_for(&app, "fillunmapped", json!({ "name": "Blank" })).await;
    let kenrith = card("Kenrith, the Returned King").type_line("Legendary Creature — Human Noble");
    seed_cards(&pool, std::slice::from_ref(&kenrith)).await;

    let commanders = vec![entry(&kenrith, "Kenrith, the Returned King")];
    let result = import(
        &pool,
        user_id,
        deck_id,
        &ArchidektDeck {
            cards: commanders.clone(),
            format: None,
            commanders,
        },
    )
    .await;

    assert_eq!(result.format_set, None);
    assert_eq!(result.commanders_set, ["Kenrith, the Returned King"]);
    let (format, zone) = profile(&pool, deck_id).await;
    assert_eq!(format, None);
    assert_eq!(zone, [Some(kenrith.id()), None, None]);
}
