//! Card metadata list endpoints (the filter-dropdown sources) + printings.
//! These are DISTINCT/tokenizing queries over `latest_cards`; a broken one
//! silently empties a filter dropdown, so a smoke test over a known fixture set
//! is worth having. All are public (no auth).
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

mod common;

use axum::http::StatusCode;
use common::{TestApp, card, seed_cards};
use uuid::Uuid;

/// The string list a metadata endpoint returns.
async fn list(app: &TestApp, path: &str) -> Vec<String> {
    let (status, body) = app.get(path, None).await;
    assert_eq!(status, StatusCode::OK, "GET {path}: {body}");
    body.as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect()
}

#[sqlx::test]
async fn metadata_endpoints_return_distinct_values(pool: sqlx::PgPool) {
    seed_cards(
        &pool,
        &[
            card("Goblin Guide")
                .type_line("Creature — Goblin")
                .keywords(&["Haste"])
                .artist("Alice Art")
                .set("M10", "Magic 2010"),
            card("Serra Angel")
                .type_line("Creature — Angel")
                .keywords(&["Flying", "Vigilance"])
                .artist("Bob Art")
                .set("M10", "Magic 2010"),
            card("Ancestral Recall")
                .type_line("Instant")
                .lang("ja")
                .artist("Alice Art"),
        ],
    )
    .await;

    let artists = list(&app_of(pool.clone()), "/api/card/artists").await;
    assert_eq!(
        artists,
        vec!["Alice Art", "Bob Art"],
        "distinct + sorted, no duplicate Alice"
    );

    let sets = list(&app_of(pool.clone()), "/api/card/sets").await;
    assert!(
        sets.contains(&"Magic 2010".to_string()) && sets.contains(&"Test Set".to_string()),
        "{sets:?}"
    );

    let langs = list(&app_of(pool.clone()), "/api/card/languages").await;
    assert!(
        langs.contains(&"en".to_string()) && langs.contains(&"ja".to_string()),
        "{langs:?}"
    );

    // keywords are lowercased + de-duped across cards
    let keywords = list(&app_of(pool.clone()), "/api/card/keywords").await;
    assert_eq!(
        keywords,
        vec!["flying", "haste", "vigilance"],
        "{keywords:?}"
    );

    // types are tokenized from type_line (stored case); stop words excluded
    let types = list(&app_of(pool.clone()), "/api/card/types").await;
    assert!(types.contains(&"Creature".to_string()), "{types:?}");
    assert!(types.contains(&"Instant".to_string()), "{types:?}");
    assert!(
        !types.iter().any(|t| t == "of" || t == "the"),
        "stop words excluded: {types:?}"
    );
}

#[sqlx::test]
async fn printings_returns_all_printings_of_an_oracle(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let shared = Uuid::from_u128(0x0B00);
    seed_cards(
        &pool,
        &[
            card("Reprinted Card")
                .oracle(Some(shared))
                .set("A", "Alpha Set"),
            card("Reprinted Card")
                .oracle(Some(shared))
                .set("B", "Beta Set"),
            card("Unrelated Card"), // its own oracle, must not appear
        ],
    )
    .await;

    let (status, body) = app
        .get(&format!("/api/card/{shared}/printings"), None)
        .await;
    assert_eq!(status, StatusCode::OK, "printings: {body}");
    let printings = body.as_array().unwrap();
    assert_eq!(printings.len(), 2, "both printings of the shared oracle");
    assert!(
        printings
            .iter()
            .all(|c| c["scryfall_data"]["name"] == "Reprinted Card"),
        "only the reprinted card: {body}"
    );
}

/// A fresh `TestApp` over a cloned pool — the metadata test issues several
/// independent public GETs and each helper call wants an app to drive.
fn app_of(pool: sqlx::PgPool) -> TestApp {
    TestApp::new(pool)
}
