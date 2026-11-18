//! The collection export against a real database: the caller's rows as a ManaBox file that the
//! collection import reads back to the same collection, cells a spreadsheet would run written as
//! text, and a collection too big to import back turned down.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![expect(
    clippy::unwrap_used,
    reason = "a test: helpers outside #[test] fns unwrap and index, and a panic is the failure report"
)]

mod common;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::atomic::{AtomicU8, Ordering};

use axum::http::{Method, StatusCode};
use chrono::{DateTime, Utc};
use common::{CardFixture, TestApp, card, seed_cards};
use serde_json::{Value, json};
use uuid::Uuid;
use zwipe_core::http::paths::{EXPORT_COLLECTION_ROUTE, IMPORT_COLLECTION_ROUTE};

static PEER: AtomicU8 = AtomicU8::new(1);

/// A fresh client address, so the import's per-address limit never trips across tests.
fn peer() -> SocketAddr {
    let n = PEER.fetch_add(1, Ordering::Relaxed);
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(13, 0, 0, n.max(1))), 42000)
}

async fn import(app: &TestApp, token: &str, csv: &str) -> Value {
    let (status, body) = app
        .send_from(
            peer(),
            Method::POST,
            IMPORT_COLLECTION_ROUTE,
            Some(json!({ "csv": csv, "mode": "replace" })),
            Some(token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "import: {body}");
    body
}

async fn export(app: &TestApp, token: Option<&str>) -> (StatusCode, Value) {
    app.send_from(peer(), Method::GET, EXPORT_COLLECTION_ROUTE, None, token)
        .await
}

/// A stored row: printing, finish, condition, language, quantity, tags, price, currency and
/// date added.
type Stored = (
    Uuid,
    String,
    Option<String>,
    Option<String>,
    i32,
    Vec<String>,
    Option<String>,
    Option<String>,
    Option<DateTime<Utc>>,
);

/// Every stored row of `user`'s collection, sorted.
async fn stored(pool: &sqlx::PgPool, user: &str) -> Vec<Stored> {
    sqlx::query_as(
        "SELECT scryfall_data_id, finish, condition, language, quantity, tags, \
                purchase_price::text, purchase_currency, added_at \
         FROM collection_cards WHERE user_id = $1 \
         ORDER BY scryfall_data_id, finish, condition NULLS FIRST, language NULLS FIRST",
    )
    .bind(user.parse::<Uuid>().unwrap())
    .fetch_all(pool)
    .await
    .unwrap()
}

/// Cards to import, each with its set and collector number.
fn cards() -> Vec<CardFixture> {
    vec![
        card("Aardwolf's Advantage")
            .with_ids(Uuid::from_u128(0xe1), Some(Uuid::from_u128(0xe1e1)))
            .set("unf", "Unfinity")
            .collector_number("97"),
        card("Abattoir Ghoul")
            .with_ids(Uuid::from_u128(0xe2), Some(Uuid::from_u128(0xe2e2)))
            .set("ddq", "Duel Decks: Blessed vs. Cursed")
            .collector_number("50"),
        card("=1+1")
            .with_ids(Uuid::from_u128(0xe3), Some(Uuid::from_u128(0xe3e3)))
            .set("tst", "Test Set")
            .collector_number("1"),
    ]
}

/// An Archidekt export of [`cards`] with every field the import keeps, a formula-looking tag
/// among them.
const ARCHIDEKT: &str = "Quantity,Name,Finish,Condition,Date Added,Language,Purchase Price,Tags,Edition Name,Edition Code,Multiverse Id,Scryfall ID,Collector Number
2,Aardwolf's Advantage,Foil,NM,2025-03-03,EN,1.25,\"trade, binder 2\",Unfinity,unf,580777,00000000-0000-0000-0000-0000000000e1,97
3,Abattoir Ghoul,Etched,LP,2025-03-04,JP,,,Duel Decks: Blessed vs. Cursed,ddq,409626,00000000-0000-0000-0000-0000000000e2,50
1,=1+1,Normal,,,,,\"@risky, =SUM(A1)\",Test Set,tst,1,00000000-0000-0000-0000-0000000000e3,1
";

#[sqlx::test]
async fn an_export_imports_back_to_the_same_collection(pool: sqlx::PgPool) {
    seed_cards(&pool, &cards()).await;
    let app = TestApp::new(pool.clone());
    let (alice, alice_id) = app.register("exporter").await;
    let (bob, bob_id) = app.register("importer").await;
    import(&app, &alice, ARCHIDEKT).await;

    let (status, file) = export(&app, Some(&alice)).await;
    assert_eq!(status, StatusCode::OK, "{file}");
    let name = file["file_name"].as_str().unwrap();
    assert!(
        name.starts_with("zwipe-collection-")
            && std::path::Path::new(name)
                .extension()
                .is_some_and(|e| e == "csv"),
        "{name}"
    );
    let csv = file["csv"].as_str().unwrap();
    assert!(
        csv.starts_with("Name,Set code,Set name,Collector number,Foil,Rarity,Quantity,ManaBox ID,Scryfall ID,Purchase price,Misprint,Altered,Signed,Condition,Language,Proxy,Purchase price currency,Added,Tags\n"),
        "ManaBox's header, then Tags: {csv}"
    );

    let summary = import(&app, &bob, csv).await;
    assert_eq!(
        summary["source"], "mana_box",
        "it reads as a ManaBox file: {summary}"
    );
    assert_eq!(summary["unmatched_total"], 0, "{summary}");

    let mut alices = stored(&pool, &alice_id).await;
    let bobs = stored(&pool, &bob_id).await;
    assert_eq!(alices.len(), 3);
    // The Tags cell started with a formula character, so it left with a ' in front: its first
    // tag comes back with it. The cell's other tags were already text.
    let risky = alices
        .iter_mut()
        .find(|row| row.0 == Uuid::from_u128(0xe3))
        .unwrap();
    assert_eq!(risky.5, ["@risky", "=SUM(A1)"]);
    risky.5 = vec!["'@risky".to_owned(), "=SUM(A1)".to_owned()];
    assert_eq!(bobs, alices, "every field made the trip");
}

#[sqlx::test]
async fn a_cell_a_spreadsheet_would_run_leaves_as_text(pool: sqlx::PgPool) {
    seed_cards(&pool, &cards()).await;
    let app = TestApp::new(pool);
    let (token, _) = app.register("formulas").await;
    import(&app, &token, ARCHIDEKT).await;

    let (_, file) = export(&app, Some(&token)).await;
    let csv = file["csv"].as_str().unwrap();
    let mut reader = csv::Reader::from_reader(csv.as_bytes());
    for record in reader.records() {
        let record = record.unwrap();
        for cell in &record {
            assert!(
                !cell.starts_with(['=', '+', '-', '@', '\t', '\r']),
                "{cell:?} in {csv}"
            );
        }
    }
    assert!(csv.contains("'=1+1"), "{csv}");
    assert!(csv.contains("'@risky"), "{csv}");
}

#[sqlx::test]
async fn an_export_holds_only_the_callers_rows(pool: sqlx::PgPool) {
    seed_cards(&pool, &cards()).await;
    let app = TestApp::new(pool);
    let (alice, _) = app.register("holder").await;
    let (bob, _) = app.register("empty").await;
    import(&app, &alice, ARCHIDEKT).await;

    let (status, file) = export(&app, Some(&bob)).await;
    assert_eq!(status, StatusCode::OK, "{file}");
    assert_eq!(
        file["csv"].as_str().unwrap().lines().count(),
        1,
        "the header alone"
    );

    let (status, _) = export(&app, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn a_collection_too_big_to_import_back_is_turned_down(pool: sqlx::PgPool) {
    // 90 printings in every finish, condition and language: 90 x 3 x 11 x 19 = 56,430 rows.
    let printings: Vec<CardFixture> = (0u128..90)
        .map(|i| {
            card(&format!("Bulk {i}")).with_ids(
                Uuid::from_u128(0xb000_u128.checked_add(i).unwrap()),
                Some(Uuid::from_u128(0xbb00_u128.checked_add(i).unwrap())),
            )
        })
        .collect();
    seed_cards(&pool, &printings).await;
    let app = TestApp::new(pool.clone());
    let (token, user) = app.register("hoarder").await;
    sqlx::query(
        "INSERT INTO collection_cards \
             (user_id, scryfall_data_id, oracle_id, finish, condition, language, quantity) \
         SELECT $1, sd.id, sd.oracle_id, f, c, l, 1 FROM scryfall_data sd \
         CROSS JOIN unnest(ARRAY['nonfoil', 'foil', 'etched']) AS f \
         CROSS JOIN unnest(ARRAY[NULL, 'mint', 'near_mint', 'excellent', 'good', 'lightly_played', \
             'moderately_played', 'played', 'heavily_played', 'poor', 'damaged']) AS c \
         CROSS JOIN unnest(ARRAY[NULL, 'en', 'es', 'fr', 'de', 'it', 'pt', 'ja', 'ko', 'ru', \
             'zhs', 'zht', 'he', 'la', 'grc', 'ar', 'sa', 'ph', 'qya']) AS l \
         WHERE sd.name LIKE 'Bulk %'",
    )
    .bind(user.parse::<Uuid>().unwrap())
    .execute(&pool)
    .await
    .unwrap();

    let (status, body) = export(&app, Some(&token)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
}
