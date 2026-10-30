//! Deck-card operations through the real router: add, quantity delta, board
//! placement, remove, and text import (resolved + unresolved). They live apart from `deck_flows.rs` because they
//! need real `cards` rows, which the `card()` / `seed_cards()` fixture builder
//! supplies.
//!
//! Note the create route is `POST /api/deck/{id}/card`, with no trailing slash
//! (the nested `/` leaf resolves without one, same as `/api/deck`).
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "a test: helpers outside #[test] fns unwrap and index, and a panic is the failure report"
)]

use zwipe_core::http::paths::{
    DECK_ROUTE, clone_deck_route, create_deck_card_route, deck_route, import_deck_cards_route,
    update_deck_card_route,
};
mod common;

use axum::http::StatusCode;
use common::{TestApp, card, seed_cards};
use serde_json::json;

/// Creates a verified user + a commander deck, returns `(token, deck_id)`.
async fn deck_for(app: &TestApp, username: &str) -> (String, String) {
    let (token, uid) = app.register(username).await;
    app.verify_email(&uid).await;
    let (status, deck) = app
        .post(
            DECK_ROUTE,
            json!({ "name": "Test Deck", "format": "commander" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "deck create: {deck}");
    (token, deck["id"].as_str().unwrap().to_string())
}

#[sqlx::test]
async fn deck_card_add_bump_remove(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, did) = deck_for(&app, "builder").await;

    let bolt = card("Lightning Bolt")
        .mono("R")
        .cmc(1.0)
        .type_line("Instant");
    let sid = bolt.id();
    let oid = bolt.oracle_id().unwrap();
    seed_cards(&pool, &[bolt]).await;

    // add one copy
    let (status, dc) = app
        .post(
            &create_deck_card_route(did.parse().unwrap()),
            json!({ "scryfall_data_id": sid.to_string(), "oracle_id": oid.to_string(), "quantity": 1 }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "add card: {dc}");
    assert_eq!(dc["quantity"], 1);
    assert_eq!(dc["board"], "deck");

    // quantity is absolute: set to 3
    let (status, dc) = app
        .patch(
            &update_deck_card_route(did.parse().unwrap(), sid),
            json!({ "quantity": 3 }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "set qty: {dc}");
    assert_eq!(dc["quantity"], 3);

    // PUT retired with the version gate: the delta route is gone.
    let (status, _) = app
        .put(
            &update_deck_card_route(did.parse().unwrap(), sid),
            json!({ "update_quantity": 2 }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED, "PUT must be gone");

    // Explicit null on a non-clearable field is a 422, not a silent no-op.
    let (status, body) = app
        .patch(
            &update_deck_card_route(did.parse().unwrap(), sid),
            json!({ "quantity": null, "board": "deck" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "null qty: {body}");

    // the full deck reflects the card + quantity
    let (_, full) = app
        .get(&deck_route(did.parse().unwrap()), Some(&token))
        .await;
    let entries = full["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 1, "one entry expected: {full}");
    assert_eq!(
        entries[0]["card"]["scryfall_data"]["name"],
        "Lightning Bolt"
    );
    assert_eq!(entries[0]["deck_card"]["quantity"], 3);

    // remove => 204, deck empties
    let (status, _) = app
        .delete(
            &update_deck_card_route(did.parse().unwrap(), sid),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT, "delete card");
    let (_, full) = app
        .get(&deck_route(did.parse().unwrap()), Some(&token))
        .await;
    assert_eq!(
        full["entries"].as_array().unwrap().len(),
        0,
        "deck should be empty after remove"
    );
}

#[sqlx::test]
async fn deck_card_added_to_maybeboard(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, did) = deck_for(&app, "mayber").await;

    let counsel = card("Counterspell").mono("U").cmc(2.0).type_line("Instant");
    let sid = counsel.id();
    let oid = counsel.oracle_id().unwrap();
    seed_cards(&pool, &[counsel]).await;

    let (status, dc) = app
        .post(
            &create_deck_card_route(did.parse().unwrap()),
            json!({
                "scryfall_data_id": sid.to_string(),
                "oracle_id": oid.to_string(),
                "quantity": 1,
                "board": "maybeboard"
            }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "add to maybeboard: {dc}");
    assert_eq!(dc["board"], "maybeboard");

    let (_, full) = app
        .get(&deck_route(did.parse().unwrap()), Some(&token))
        .await;
    let entries = full["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["deck_card"]["board"], "maybeboard");
}

#[sqlx::test]
async fn clone_copies_the_cards(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, did) = deck_for(&app, "cloner").await;

    let a = card("Sol Ring").color_identity("").type_line("Artifact");
    let b = card("Arcane Signet")
        .color_identity("")
        .type_line("Artifact");
    let (a_sid, a_oid) = (a.id(), a.oracle_id().unwrap());
    let (b_sid, b_oid) = (b.id(), b.oracle_id().unwrap());
    seed_cards(&pool, &[a, b]).await;

    for (sid, oid, qty) in [(a_sid, a_oid, 1), (b_sid, b_oid, 3)] {
        let (status, _) = app
            .post(
                &create_deck_card_route(did.parse().unwrap()),
                json!({ "scryfall_data_id": sid.to_string(), "oracle_id": oid.to_string(), "quantity": qty }),
                Some(&token),
            )
            .await;
        assert_eq!(status, StatusCode::CREATED);
    }

    let (status, cloned) = app
        .post(
            &clone_deck_route(did.parse().unwrap()),
            json!({ "new_name": "Sol Ring Copy" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "clone: {cloned}");
    let clone_id = cloned["deck_id"].as_str().unwrap();

    // the clone carries both cards with their quantities
    let (_, full) = app
        .get(&deck_route(clone_id.parse().unwrap()), Some(&token))
        .await;
    let entries = full["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 2, "clone should copy both cards: {full}");
    let mut by_name: Vec<(String, i64)> = entries
        .iter()
        .map(|e| {
            (
                e["card"]["scryfall_data"]["name"]
                    .as_str()
                    .unwrap()
                    .to_string(),
                e["deck_card"]["quantity"].as_i64().unwrap(),
            )
        })
        .collect();
    by_name.sort();
    assert_eq!(
        by_name,
        vec![
            ("Arcane Signet".to_string(), 3),
            ("Sol Ring".to_string(), 1)
        ]
    );
}

#[sqlx::test]
async fn import_resolves_known_cards_and_reports_the_rest(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, did) = deck_for(&app, "importer").await;

    seed_cards(
        &pool,
        &[
            card("Lightning Bolt").mono("R").type_line("Instant"),
            card("Llanowar Elves")
                .mono("G")
                .type_line("Creature — Elf Druid"),
        ],
    )
    .await;

    // one resolvable, one bogus line
    let (status, result) = app
        .post(
            &import_deck_cards_route(did.parse().unwrap()),
            json!({ "text": "2 Lightning Bolt\n1 Definitely Not A Real Card" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "import: {result}");
    let imported = result["imported"].as_array().unwrap();
    let unresolved = result["unresolved"].as_array().unwrap();
    assert_eq!(imported.len(), 1, "one card should resolve: {result}");
    assert_eq!(
        unresolved.len(),
        1,
        "the bogus line should be unresolved: {result}"
    );

    // the imported card really landed in the deck
    let (_, full) = app
        .get(&deck_route(did.parse().unwrap()), Some(&token))
        .await;
    let names: Vec<&str> = full["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["card"]["scryfall_data"]["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, vec!["Lightning Bolt"]);
}

/// `GET /api/deck/{id}` carries the command-zone cards (commander, partner, …)
/// so clients can fold them into price and card-count calcs. Regression guard
/// for the `Deck.command_zone_cards` field + its server population.
#[sqlx::test]
async fn get_deck_carries_command_zone_cards(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, did) = deck_for(&app, "czbuilder").await;

    // A priced commander (command zone) and a priced mainboard card.
    let commander = card("Atraxa, Praetors' Voice")
        .colors("WUBG")
        .cmc(4.0)
        .type_line("Legendary Creature — Phyrexian Angel Horror")
        .commander_legal()
        .usd("10.00");
    let cmd_sid = commander.id();
    let bolt = card("Lightning Bolt")
        .mono("R")
        .cmc(1.0)
        .type_line("Instant")
        .usd("2.00");
    let bolt_sid = bolt.id();
    let bolt_oid = bolt.oracle_id().unwrap();
    seed_cards(&pool, &[commander, bolt]).await;

    // Set the commander on the deck (clean Patch wire: bare value = set,
    // absent = unchanged; the tagged dialect is not decoded).
    let (status, updated) = app
        .patch(
            &deck_route(did.parse().unwrap()),
            json!({ "commander_id": cmd_sid.to_string() }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "set commander: {updated}");

    // Add the mainboard card.
    let (status, dc) = app
        .post(
            &create_deck_card_route(did.parse().unwrap()),
            json!({ "scryfall_data_id": bolt_sid.to_string(), "oracle_id": bolt_oid.to_string(), "quantity": 1 }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "add bolt: {dc}");

    let (status, full) = app
        .get(&deck_route(did.parse().unwrap()), Some(&token))
        .await;
    assert_eq!(status, StatusCode::OK, "get deck: {full}");

    // Command zone carries the commander (with its price), separate from entries.
    let cz = full["command_zone_cards"].as_array().unwrap();
    assert_eq!(cz.len(), 1, "one command-zone card: {full}");
    assert_eq!(cz[0]["scryfall_data"]["name"], "Atraxa, Praetors' Voice");
    assert_eq!(cz[0]["scryfall_data"]["prices"]["usd"], "10.00");

    // The commander is NOT duplicated into the mainboard entries.
    let entries = full["entries"].as_array().unwrap();
    assert_eq!(
        entries.len(),
        1,
        "only the mainboard card in entries: {full}"
    );
    assert_eq!(
        entries[0]["card"]["scryfall_data"]["name"],
        "Lightning Bolt"
    );
}

/// The server takes a card's `oracle_id` from its own `scryfall_data` row and
/// ignores the client's, so a forged one can't slip the commander or a second
/// copy of a singleton past the checks, and a printing swap can't change the
/// card.
#[sqlx::test]
async fn oracle_id_comes_from_the_printing_not_the_client(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, did) = deck_for(&app, "forger").await;
    let deck_id = did.parse().unwrap();

    let commander = card("Atraxa, Praetors' Voice")
        .colors("WUBG")
        .type_line("Legendary Creature — Phyrexian Angel Horror")
        .commander_legal();
    let cmd_sid = commander.id();
    let sol = card("Sol Ring").color_identity("").type_line("Artifact");
    let (sol_sid, sol_oid) = (sol.id(), sol.oracle_id().unwrap());
    let sol_reprint = card("Sol Ring")
        .color_identity("")
        .type_line("Artifact")
        .oracle(Some(sol_oid));
    let reprint_sid = sol_reprint.id();
    let bolt = card("Lightning Bolt").mono("R").type_line("Instant");
    let bolt_sid = bolt.id();
    let forged_oid = uuid::Uuid::new_v4();
    seed_cards(&pool, &[commander, sol, sol_reprint, bolt]).await;

    let (status, body) = app
        .patch(
            &deck_route(deck_id),
            json!({ "commander_id": cmd_sid.to_string() }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "set commander: {body}");

    // The commander's own printing with someone else's oracle id.
    let (status, body) = app
        .post(
            &create_deck_card_route(deck_id),
            json!({ "scryfall_data_id": cmd_sid.to_string(), "oracle_id": forged_oid.to_string(), "quantity": 1 }),
            Some(&token),
        )
        .await;
    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "commander under a forged oracle id: {body}"
    );

    // A second Sol Ring printing under a forged oracle id.
    let (status, body) = app
        .post(
            &create_deck_card_route(deck_id),
            json!({ "scryfall_data_id": sol_sid.to_string(), "oracle_id": sol_oid.to_string(), "quantity": 1 }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "add sol ring: {body}");
    let (status, body) = app
        .post(
            &create_deck_card_route(deck_id),
            json!({ "scryfall_data_id": reprint_sid.to_string(), "oracle_id": forged_oid.to_string(), "quantity": 1 }),
            Some(&token),
        )
        .await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "second sol ring under a forged oracle id: {body}"
    );

    // A wrong oracle id on an honest printing is ignored, not stored.
    let (status, body) = app
        .post(
            &create_deck_card_route(deck_id),
            json!({ "scryfall_data_id": bolt_sid.to_string(), "oracle_id": forged_oid.to_string(), "quantity": 1 }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "add bolt: {body}");
    let stored: uuid::Uuid = sqlx::query_scalar(
        "SELECT oracle_id FROM deck_cards WHERE deck_id = $1 AND scryfall_data_id = $2",
    )
    .bind(deck_id)
    .bind(bolt_sid)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_ne!(stored, forged_oid, "the forged oracle id was stored");

    // Swapping Sol Ring's printing to a different card is refused.
    let (status, body) = app
        .patch(
            &update_deck_card_route(deck_id, sol_sid),
            json!({ "scryfall_data_id": cmd_sid.to_string() }),
            Some(&token),
        )
        .await;
    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "printing swap to another card: {body}"
    );

    // Swapping to another printing of the same card still works.
    let (status, body) = app
        .patch(
            &update_deck_card_route(deck_id, sol_sid),
            json!({ "scryfall_data_id": reprint_sid.to_string() }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "printing swap to a reprint: {body}");
}

/// Two stars racing onto a deck with two MVPs end at the cap of three: the
/// star count runs under the deck row lock.
#[sqlx::test]
async fn concurrent_stars_stop_at_the_mvp_cap(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, did) = deck_for(&app, "starrer").await;
    let deck_id = did.parse().unwrap();

    let cards = [
        card("Sol Ring").type_line("Artifact"),
        card("Arcane Signet").type_line("Artifact"),
        card("Mind Stone").type_line("Artifact"),
        card("Fellwar Stone").type_line("Artifact"),
    ];
    let ids: Vec<(uuid::Uuid, uuid::Uuid)> = cards
        .iter()
        .map(|c| (c.id(), c.oracle_id().unwrap()))
        .collect();
    seed_cards(&pool, &cards).await;
    for (sid, oid) in &ids {
        let (status, body) = app
            .post(
                &create_deck_card_route(deck_id),
                json!({ "scryfall_data_id": sid.to_string(), "oracle_id": oid.to_string(), "quantity": 1 }),
                Some(&token),
            )
            .await;
        assert_eq!(status, StatusCode::CREATED, "add card: {body}");
    }
    for (sid, _) in &ids[..2] {
        let (status, body) = app
            .patch(
                &update_deck_card_route(deck_id, *sid),
                json!({ "mvp": true }),
                Some(&token),
            )
            .await;
        assert_eq!(status, StatusCode::OK, "star: {body}");
    }

    // Hold the deck card rows so both requests count the stars before
    // either writes.
    let hold = async {
        let mut tx = pool.begin().await.unwrap();
        sqlx::query("SELECT 1 FROM deck_cards WHERE deck_id = $1 FOR UPDATE")
            .bind(deck_id)
            .execute(&mut *tx)
            .await
            .unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        tx.commit().await.unwrap();
    };
    let star = json!({ "mvp": true });
    let (third, fourth) = (
        update_deck_card_route(deck_id, ids[2].0),
        update_deck_card_route(deck_id, ids[3].0),
    );
    let ((first, _), (second, _), ()) = tokio::join!(
        app.patch(&third, star.clone(), Some(&token)),
        app.patch(&fourth, star.clone(), Some(&token)),
        hold,
    );
    let starred: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM deck_cards WHERE deck_id = $1 AND mvp_at IS NOT NULL",
    )
    .bind(deck_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(starred, 3, "statuses {first} and {second}");
}

/// The import result lists its cards by name, whatever order the text gave.
#[sqlx::test]
async fn import_result_is_sorted_by_name(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, did) = deck_for(&app, "sorter").await;

    let names = [
        "Wrath of God",
        "Brainstorm",
        "Ponder",
        "Counterspell",
        "Swords to Plowshares",
        "Arcane Signet",
        "Lightning Bolt",
        "Duress",
    ];
    let cards: Vec<_> = names
        .iter()
        .map(|name| card(name).type_line("Instant"))
        .collect();
    seed_cards(&pool, &cards).await;

    let text = names.map(|name| format!("1 {name}")).join("\n");
    let (status, result) = app
        .post(
            &import_deck_cards_route(did.parse().unwrap()),
            json!({ "text": text }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "import: {result}");
    let imported: Vec<&str> = result["imported"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap())
        .collect();
    let mut sorted = names.to_vec();
    sorted.sort_unstable();
    assert_eq!(imported, sorted);
}
