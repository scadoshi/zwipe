//! zervice's removal of the printings Scryfall deleted, against a real database
//! (`zwipe-context ideas/sync_remove_deleted_printings.md`): a preview changes nothing, an
//! applied run moves every deck reference to the card's cheapest regular survivor before it
//! deletes, keeps a held card with no other printing, and a second run finds nothing to do. An id
//! an older client still holds is translated by the deck routes.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "a test: helpers outside #[test] fns unwrap and index, and a panic is the failure report"
)]

mod common;

use std::collections::HashSet;

use axum::http::StatusCode;
use common::{CardFixture, TestApp, card, seed_cards};
use serde_json::json;
use uuid::Uuid;
use zwipe::domain::card;
use zwipe::domain::card::models::printing_removal::{RemovalCounts, RemovalMode};
use zwipe::domain::card::ports::CardService;
use zwipe::outbound::sqlx::postgres::Postgres;
use zwipe_core::http::paths::{DECK_ROUTE, update_deck_card_route, update_deck_route};

/// The fixture's printings, by what each one tests.
struct Printings {
    /// Ashling's deleted promo, held by the deck (two copies, starred) and as its commander.
    promo: CardFixture,
    /// Ashling's cheapest regular survivor: the replacement.
    regular: CardFixture,
    /// Cheaper, but its USD price is under a tenth of its EUR price (#47): skipped.
    bad_price: CardFixture,
    /// Cheapest of all, but a promo: not regular.
    cheap_promo: CardFixture,
    /// A deleted printing of a card whose only survivor is a promo: it falls back to that.
    lone: CardFixture,
    /// That card's promo survivor.
    lone_promo: CardFixture,
    /// A deleted printing of a card with no other printing that nothing holds: removed.
    unheld: CardFixture,
    /// A deleted printing of a card with no other printing that the deck holds: kept.
    held: CardFixture,
    /// A printing still in the bulk file.
    kept_in_bulk: CardFixture,
}

fn printings() -> Printings {
    let ashling = Uuid::new_v4();
    let lone = Uuid::new_v4();
    let promo = card("Ashling, Rekindled")
        .oracle(Some(ashling))
        .set("pecl", "Lorwyn Eclipsed Promos")
        .collector_number("124p")
        .promo(true)
        .usd("4.00");
    let regular = card("Ashling, Rekindled")
        .oracle(Some(ashling))
        .set("ecl", "Lorwyn Eclipsed")
        .usd("0.50");
    let bad_price = card("Ashling, Rekindled")
        .oracle(Some(ashling))
        .set("ecl", "Lorwyn Eclipsed")
        .usd("0.30")
        .eur("5.00");
    let cheap_promo = card("Ashling, Rekindled")
        .oracle(Some(ashling))
        .set("pecl", "Lorwyn Eclipsed Promos")
        .promo(true)
        .usd("0.10");
    Printings {
        promo,
        regular,
        bad_price,
        cheap_promo,
        lone: card("Lone Card")
            .oracle(Some(lone))
            .set("abc", "Alpha")
            .usd("1.00"),
        lone_promo: card("Lone Card")
            .oracle(Some(lone))
            .set("pabc", "Alpha Promos")
            .promo(true),
        unheld: card("Unheld Card").set("xyz", "Gone"),
        held: card("Held Card").set("xyz", "Gone"),
        kept_in_bulk: card("Still Here").set("lea", "Alpha"),
    }
}

impl Printings {
    fn all(&self) -> Vec<CardFixture> {
        vec![
            self.promo.clone(),
            self.regular.clone(),
            self.bad_price.clone(),
            self.cheap_promo.clone(),
            self.lone.clone(),
            self.lone_promo.clone(),
            self.unheld.clone(),
            self.held.clone(),
            self.kept_in_bulk.clone(),
        ]
    }

    /// The bulk file's ids: every printing but the four Scryfall deleted.
    fn bulk(&self) -> HashSet<Uuid> {
        [
            &self.regular,
            &self.bad_price,
            &self.cheap_promo,
            &self.lone_promo,
            &self.kept_in_bulk,
        ]
        .into_iter()
        .map(CardFixture::id)
        .collect()
    }
}

/// Seeds the printings and a deck holding the promo (two starred copies, and as commander), the
/// lone card (maybeboard, and as partner) and the held card. Returns the app, its token and the
/// deck id.
async fn seeded(pool: sqlx::PgPool, p: &Printings) -> (TestApp, String, Uuid) {
    seed_cards(&pool, &p.all()).await;
    let app = TestApp::new(pool);
    let (token, uid) = app.register("remover").await;
    app.verify_email(&uid).await;
    let (status, deck) = app
        .post(
            DECK_ROUTE,
            json!({ "name": "Ashling", "format": "commander" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "deck create: {deck}");
    let deck_id: Uuid = deck["id"].as_str().unwrap().parse().unwrap();
    for (printing, quantity, board, mvp) in [
        (&p.promo, 2, "deck", true),
        (&p.lone, 1, "maybeboard", false),
        (&p.held, 1, "deck", false),
    ] {
        sqlx::query(
            "INSERT INTO deck_cards (deck_id, scryfall_data_id, oracle_id, quantity, board, mvp_at) \
             VALUES ($1, $2, $3, $4, $5, CASE WHEN $6 THEN now() END)",
        )
        .bind(deck_id)
        .bind(printing.id())
        .bind(printing.oracle_id())
        .bind(quantity)
        .bind(board)
        .bind(mvp)
        .execute(&app.pool)
        .await
        .unwrap();
    }
    sqlx::query("UPDATE decks SET commander_id = $2, partner_commander_id = $3 WHERE id = $1")
        .bind(deck_id)
        .bind(p.promo.id())
        .bind(p.lone.id())
        .execute(&app.pool)
        .await
        .unwrap();
    (app, token, deck_id)
}

fn service(pool: &sqlx::PgPool) -> card::services::Service<Postgres> {
    card::services::Service::new(Postgres::from_pool(pool.clone()))
}

/// The deck's cards as `(printing, quantity, board, starred)`, by printing.
async fn deck_cards(pool: &sqlx::PgPool, deck_id: Uuid) -> Vec<(Uuid, i32, String, bool)> {
    sqlx::query_as(
        "SELECT scryfall_data_id, quantity, board, mvp_at IS NOT NULL FROM deck_cards \
         WHERE deck_id = $1 ORDER BY scryfall_data_id",
    )
    .bind(deck_id)
    .fetch_all(pool)
    .await
    .unwrap()
}

/// The deck's commander and partner.
async fn slots(pool: &sqlx::PgPool, deck_id: Uuid) -> (Option<Uuid>, Option<Uuid>) {
    sqlx::query_as("SELECT commander_id, partner_commander_id FROM decks WHERE id = $1")
        .bind(deck_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Every stored printing's id.
async fn stored(pool: &sqlx::PgPool) -> HashSet<Uuid> {
    sqlx::query_scalar::<_, Uuid>("SELECT id FROM scryfall_data")
        .fetch_all(pool)
        .await
        .unwrap()
        .into_iter()
        .collect()
}

/// The removal log as `(id, replacement)`, by id.
async fn removed(pool: &sqlx::PgPool) -> Vec<(Uuid, Option<Uuid>)> {
    sqlx::query_as("SELECT id, replacement_id FROM removed_printings ORDER BY id")
        .fetch_all(pool)
        .await
        .unwrap()
}

fn sorted(mut rows: Vec<(Uuid, i32, String, bool)>) -> Vec<(Uuid, i32, String, bool)> {
    rows.sort();
    rows
}

#[sqlx::test]
async fn a_preview_reports_the_plan_and_changes_nothing(pool: sqlx::PgPool) {
    let p = printings();
    let (app, _, deck_id) = seeded(pool, &p).await;
    let before = (
        stored(&app.pool).await,
        deck_cards(&app.pool, deck_id).await,
        slots(&app.pool, deck_id).await,
    );

    let report = service(&app.pool)
        .remove_deleted_printings(&p.bulk(), RemovalMode::Preview)
        .await
        .unwrap();

    let moves: Vec<(Uuid, Option<Uuid>)> = report
        .plan
        .removals
        .iter()
        .map(|r| (r.printing.id, r.replacement.as_ref().map(|x| x.id)))
        .collect();
    assert_eq!(
        moves,
        vec![
            (p.promo.id(), Some(p.regular.id())),
            (p.lone.id(), Some(p.lone_promo.id())),
            (p.unheld.id(), None),
        ],
        "Ashling's promo to its cheapest regular printing (not the #47-skipped one or the promo), the lone card to its only survivor, the unheld card with no survivor out"
    );
    assert_eq!(
        report
            .plan
            .kept
            .iter()
            .map(|k| k.printing.id)
            .collect::<Vec<_>>(),
        vec![p.held.id()],
        "the held card with no other printing stays"
    );
    assert_eq!(report.counts, None, "a preview applies nothing");
    let after = (
        stored(&app.pool).await,
        deck_cards(&app.pool, deck_id).await,
        slots(&app.pool, deck_id).await,
    );
    assert_eq!(before, after, "a preview changes nothing");
    assert!(removed(&app.pool).await.is_empty(), "and logs no removal");
}

#[sqlx::test]
async fn an_applied_run_remaps_every_reference_then_removes(pool: sqlx::PgPool) {
    let p = printings();
    let (app, _, deck_id) = seeded(pool, &p).await;

    let report = service(&app.pool)
        .remove_deleted_printings(&p.bulk(), RemovalMode::Apply)
        .await
        .unwrap();

    assert_eq!(
        report.counts,
        Some(RemovalCounts {
            removed: 3,
            remapped_deck_cards: 2,
            remapped_slots: 2,
        })
    );
    assert_eq!(
        sorted(deck_cards(&app.pool, deck_id).await),
        sorted(vec![
            (p.regular.id(), 2, "deck".to_owned(), true),
            (p.lone_promo.id(), 1, "maybeboard".to_owned(), false),
            (p.held.id(), 1, "deck".to_owned(), false),
        ]),
        "every card is still in the deck, with its quantity, board and star"
    );
    assert_eq!(
        slots(&app.pool, deck_id).await,
        (Some(p.regular.id()), Some(p.lone_promo.id())),
        "the commander and partner moved too"
    );
    let left = stored(&app.pool).await;
    for gone in [&p.promo, &p.lone, &p.unheld] {
        assert!(!left.contains(&gone.id()), "{} is removed", gone.name());
    }
    assert!(left.contains(&p.held.id()), "the held card is kept");
    let profiles: i64 =
        sqlx::query_scalar("SELECT count(*) FROM card_profiles WHERE scryfall_data_id = ANY($1)")
            .bind(vec![p.promo.id(), p.lone.id(), p.unheld.id()])
            .fetch_one(&app.pool)
            .await
            .unwrap();
    assert_eq!(profiles, 0, "their profiles go with them");
    let mut want = vec![
        (p.promo.id(), Some(p.regular.id())),
        (p.lone.id(), Some(p.lone_promo.id())),
        (p.unheld.id(), None),
    ];
    want.sort();
    assert_eq!(removed(&app.pool).await, want, "each removal is logged");
}

#[sqlx::test]
async fn a_second_run_finds_nothing_to_do(pool: sqlx::PgPool) {
    let p = printings();
    let (app, _, deck_id) = seeded(pool, &p).await;
    let cards = service(&app.pool);
    let _ = cards
        .remove_deleted_printings(&p.bulk(), RemovalMode::Apply)
        .await
        .unwrap();
    let after_first = (
        stored(&app.pool).await,
        deck_cards(&app.pool, deck_id).await,
        removed(&app.pool).await,
    );

    let second = cards
        .remove_deleted_printings(&p.bulk(), RemovalMode::Apply)
        .await
        .unwrap();

    assert!(
        second.plan.removals.is_empty(),
        "{:?}",
        second.plan.removals
    );
    assert_eq!(
        second
            .plan
            .kept
            .iter()
            .map(|k| k.printing.id)
            .collect::<Vec<_>>(),
        vec![p.held.id()],
        "the held card is reported again"
    );
    assert_eq!(second.counts, Some(RemovalCounts::default()));
    assert_eq!(
        (
            stored(&app.pool).await,
            deck_cards(&app.pool, deck_id).await,
            removed(&app.pool).await,
        ),
        after_first,
        "nothing changed"
    );
}

#[sqlx::test]
async fn an_id_an_older_client_still_holds_lands_on_its_replacement(pool: sqlx::PgPool) {
    let p = printings();
    let (app, token, deck_id) = seeded(pool, &p).await;
    let cards = service(&app.pool);
    let _ = cards
        .remove_deleted_printings(&p.bulk(), RemovalMode::Apply)
        .await
        .unwrap();

    // An undo naming the removed promo as the commander.
    let (status, body) = app
        .patch(
            &update_deck_route(deck_id),
            json!({ "commander_id": p.promo.id().to_string() }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        slots(&app.pool, deck_id).await.0,
        Some(p.regular.id()),
        "the profile stores the replacement"
    );

    // A quantity change on the deck card by the id the client knew it by.
    let (status, body) = app
        .patch(
            &update_deck_card_route(deck_id, p.promo.id()),
            json!({ "quantity": 3 }),
            Some(&token),
        )
        .await;
    assert!(status.is_success(), "{status} {body}");
    assert!(
        deck_cards(&app.pool, deck_id).await.contains(&(
            p.regular.id(),
            3,
            "deck".to_owned(),
            true
        )),
        "the change lands on the card's row"
    );

    // A slot written with the old id behind the API's back is moved by the next run.
    sqlx::query("UPDATE decks SET partner_commander_id = $2 WHERE id = $1")
        .bind(deck_id)
        .bind(p.lone.id())
        .execute(&app.pool)
        .await
        .unwrap();
    let next = cards
        .remove_deleted_printings(&p.bulk(), RemovalMode::Apply)
        .await
        .unwrap();
    assert_eq!(next.counts.map(|c| c.remapped_slots), Some(1));
    assert_eq!(slots(&app.pool, deck_id).await.1, Some(p.lone_promo.id()));

    // Removing the card by its old id removes its row.
    let (status, body) = app
        .delete(&update_deck_card_route(deck_id, p.promo.id()), Some(&token))
        .await;
    assert!(status.is_success(), "{status} {body}");
    assert!(
        !deck_cards(&app.pool, deck_id)
            .await
            .iter()
            .any(|row| row.0 == p.regular.id()),
        "the card's row is gone"
    );
}
