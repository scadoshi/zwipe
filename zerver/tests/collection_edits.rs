//! Editing and listing the collection against a real database: a set call gives exactly one row
//! its copies (0 removes it), and the list reads the caller's own rows a page at a time, sorted
//! by name and searchable by it. Both need a session and touch only the caller's collection.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "a test: helpers outside #[test] fns unwrap and index, and a panic is the failure report"
)]

mod common;

use axum::http::StatusCode;
use common::{CardFixture, TestApp, card, seed_cards};
use serde_json::{Value, json};
use uuid::Uuid;
use zwipe_core::http::paths::{COLLECTION_CARDS_ROUTE, COLLECTION_ROUTE};

/// A set call for `printing` in `finish`, with `extra` fields (condition, language) merged in.
fn set_call(printing: &CardFixture, finish: &str, quantity: i32, extra: &Value) -> Value {
    let mut body = json!({
        "scryfall_data_id": printing.id(),
        "finish": finish,
        "quantity": quantity,
    });
    if let (Some(body), Some(extra)) = (body.as_object_mut(), extra.as_object()) {
        for (key, value) in extra {
            body.insert(key.clone(), value.clone());
        }
    }
    body
}

/// The list's rows as `(name, finish, condition, language, quantity)`.
fn rows(page: &Value) -> Vec<(String, String, Value, Value, i64)> {
    page["cards"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            (
                row["card"]["scryfall_data"]["name"]
                    .as_str()
                    .unwrap()
                    .to_owned(),
                row["finish"].as_str().unwrap().to_owned(),
                row["condition"].clone(),
                row["language"].clone(),
                row["quantity"].as_i64().unwrap(),
            )
        })
        .collect()
}

/// The caller's whole collection, one page of up to 100.
async fn list(app: &TestApp, token: &str) -> Value {
    let (status, page) = app
        .get(
            &format!("{COLLECTION_ROUTE}?offset=0&limit=100"),
            Some(token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{page}");
    page
}

/// Puts an imported row straight in: `quantity` of `printing` in `finish`, `condition` and
/// `language`.
async fn imported(
    pool: &sqlx::PgPool,
    user: &str,
    printing: &CardFixture,
    finish: &str,
    condition: Option<&str>,
    language: Option<&str>,
    quantity: i32,
) {
    sqlx::query(
        "INSERT INTO collection_cards \
         (user_id, scryfall_data_id, oracle_id, finish, condition, language, quantity) \
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(user.parse::<Uuid>().unwrap())
    .bind(printing.id())
    .bind(printing.oracle_id().unwrap())
    .bind(finish)
    .bind(condition)
    .bind(language)
    .bind(quantity)
    .execute(pool)
    .await
    .unwrap();
}

#[sqlx::test]
async fn a_set_call_adds_changes_and_removes_the_manual_row(pool: sqlx::PgPool) {
    let bolt = card("Lightning Bolt").mono("R");
    seed_cards(&pool, std::slice::from_ref(&bolt)).await;
    let app = TestApp::new(pool);
    let (token, _) = app.register("setter").await;

    let (status, reply) = app
        .put(
            COLLECTION_CARDS_ROUTE,
            set_call(&bolt, "foil", 2, &json!({})),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{reply}");
    assert_eq!(reply["card"]["quantity"], 2);
    assert_eq!(reply["card"]["finish"], "foil");
    assert_eq!(reply["card"]["condition"], Value::Null);
    assert_eq!(
        reply["card"]["card"]["scryfall_data"]["name"],
        "Lightning Bolt"
    );

    let (status, reply) = app
        .put(
            COLLECTION_CARDS_ROUTE,
            set_call(&bolt, "foil", 5, &json!({})),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{reply}");
    assert_eq!(reply["card"]["quantity"], 5);
    assert_eq!(
        rows(&list(&app, &token).await),
        vec![(
            "Lightning Bolt".to_owned(),
            "foil".to_owned(),
            Value::Null,
            Value::Null,
            5
        )],
        "the same row, changed"
    );

    let (status, reply) = app
        .put(
            COLLECTION_CARDS_ROUTE,
            set_call(&bolt, "foil", 0, &json!({})),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{reply}");
    assert_eq!(reply["card"], Value::Null, "0 removes it");
    assert_eq!(rows(&list(&app, &token).await), vec![]);

    let (status, reply) = app
        .put(
            COLLECTION_CARDS_ROUTE,
            set_call(&bolt, "foil", 0, &json!({})),
            Some(&token),
        )
        .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "removing a row that isn't there is no error: {reply}"
    );
    assert_eq!(reply["card"], Value::Null);
}

#[sqlx::test]
async fn a_set_call_edits_exactly_the_row_it_names(pool: sqlx::PgPool) {
    let bolt = card("Lightning Bolt").mono("R");
    seed_cards(&pool, std::slice::from_ref(&bolt)).await;
    let app = TestApp::new(pool.clone());
    let (token, user) = app.register("tapper").await;
    imported(
        &pool,
        &user,
        &bolt,
        "nonfoil",
        Some("near_mint"),
        Some("ja"),
        3,
    )
    .await;

    let named = json!({ "condition": "near_mint", "language": "ja" });
    let (status, reply) = app
        .put(
            COLLECTION_CARDS_ROUTE,
            set_call(&bolt, "nonfoil", 4, &named),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{reply}");
    assert_eq!(reply["card"]["quantity"], 4);
    assert_eq!(reply["card"]["condition"], "near_mint");
    assert_eq!(
        rows(&list(&app, &token).await),
        vec![(
            "Lightning Bolt".to_owned(),
            "nonfoil".to_owned(),
            json!("near_mint"),
            json!("ja"),
            4
        )],
        "the tapped row changed and no second row came"
    );

    let (status, _) = app
        .put(
            COLLECTION_CARDS_ROUTE,
            set_call(&bolt, "nonfoil", 1, &json!({})),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        rows(&list(&app, &token).await).len(),
        2,
        "with no condition or language it's the manual row, beside the imported one"
    );
}

#[sqlx::test]
async fn one_user_never_sees_or_touches_anothers_rows(pool: sqlx::PgPool) {
    let bolt = card("Lightning Bolt").mono("R");
    seed_cards(&pool, std::slice::from_ref(&bolt)).await;
    let app = TestApp::new(pool.clone());
    let (alice, alice_id) = app.register("alice").await;
    let (bob, _) = app.register("bobby").await;
    imported(&pool, &alice_id, &bolt, "foil", None, None, 3).await;

    let (status, reply) = app
        .put(
            COLLECTION_CARDS_ROUTE,
            set_call(&bolt, "foil", 0, &json!({})),
            Some(&bob),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{reply}");
    let (status, _) = app
        .put(
            COLLECTION_CARDS_ROUTE,
            set_call(&bolt, "foil", 7, &json!({})),
            Some(&bob),
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    assert_eq!(
        rows(&list(&app, &alice).await),
        vec![(
            "Lightning Bolt".to_owned(),
            "foil".to_owned(),
            Value::Null,
            Value::Null,
            3
        )],
        "Alice's row is as it was"
    );
    assert_eq!(
        rows(&list(&app, &bob).await),
        vec![(
            "Lightning Bolt".to_owned(),
            "foil".to_owned(),
            Value::Null,
            Value::Null,
            7
        )],
        "Bob has his own row"
    );
}

#[sqlx::test]
async fn a_set_call_refuses_what_it_cant_store(pool: sqlx::PgPool) {
    let bolt = card("Lightning Bolt").mono("R");
    let art = card("Lightning Bolt").with_ids(Uuid::from_u128(0x5a7), None);
    seed_cards(&pool, &[bolt.clone(), art.clone()]).await;
    let app = TestApp::new(pool);
    let (token, _) = app.register("refused").await;

    let unknown = json!({
        "scryfall_data_id": Uuid::from_u128(0xdead),
        "finish": "foil",
        "quantity": 1,
    });
    let (status, body) = app.put(COLLECTION_CARDS_ROUTE, unknown, Some(&token)).await;
    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "no such printing: {body}"
    );
    let (status, body) = app
        .put(
            COLLECTION_CARDS_ROUTE,
            set_call(&art, "foil", 1, &json!({})),
            Some(&token),
        )
        .await;
    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "a printing with no card: {body}"
    );

    for (field, value) in [
        ("finish", "gilded"),
        ("condition", "pristine"),
        ("language", "klingon"),
    ] {
        let mut body = set_call(&bolt, "foil", 1, &json!({}));
        body[field] = json!(value);
        let (status, reply) = app.put(COLLECTION_CARDS_ROUTE, body, Some(&token)).await;
        assert_eq!(
            status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "{field} {value} reads as Unknown and is refused: {reply}"
        );
    }
    for quantity in [-1, 10_000] {
        let (status, _) = app
            .put(
                COLLECTION_CARDS_ROUTE,
                set_call(&bolt, "foil", quantity, &json!({})),
                Some(&token),
            )
            .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{quantity}");
    }
    assert_eq!(
        rows(&list(&app, &token).await),
        vec![],
        "nothing was stored"
    );

    let (status, _) = app
        .put(
            COLLECTION_CARDS_ROUTE,
            set_call(&bolt, "foil", 1, &json!({})),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = app.get(COLLECTION_ROUTE, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn the_list_pages_by_name_and_searches_names_as_text(pool: sqlx::PgPool) {
    let sol = Uuid::from_u128(0x501);
    let printings = [
        card("Sol Ring")
            .with_ids(Uuid::from_u128(0x5011), Some(sol))
            .set("c21", "Commander 2021"),
        card("Sol Ring")
            .with_ids(Uuid::from_u128(0x5012), Some(sol))
            .set("c13", "Commander 2013"),
        card("Lightning Bolt").mono("R"),
        card("Lim-Dûl's Vault"),
        card("100% Ring"),
    ];
    seed_cards(&pool, &printings).await;
    let app = TestApp::new(pool.clone());
    let (token, user) = app.register("pager").await;
    for printing in &printings {
        imported(&pool, &user, printing, "nonfoil", None, None, 1).await;
    }

    let (status, first) = app
        .get(
            &format!("{COLLECTION_ROUTE}?offset=0&limit=2"),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{first}");
    assert_eq!(first["total"], 5);
    assert_eq!(first["next_offset"], 2);
    let (_, second) = app
        .get(
            &format!("{COLLECTION_ROUTE}?offset=2&limit=2"),
            Some(&token),
        )
        .await;
    let (_, last) = app
        .get(
            &format!("{COLLECTION_ROUTE}?offset=4&limit=2"),
            Some(&token),
        )
        .await;
    assert_eq!(last["next_offset"], Value::Null, "the last page");
    let names: Vec<String> = [first, second, last]
        .iter()
        .flat_map(|page| rows(page).into_iter().map(|row| row.0))
        .collect();
    assert_eq!(
        names,
        [
            "100% Ring",
            "Lightning Bolt",
            "Lim-Dûl's Vault",
            "Sol Ring",
            "Sol Ring"
        ],
        "sorted by name"
    );

    let search = |name: &str| format!("{COLLECTION_ROUTE}?name={name}&offset=0&limit=100");
    let (_, sol_rows) = app.get(&search("ring"), Some(&token)).await;
    assert_eq!(
        sol_rows["total"], 3,
        "Sol Ring twice and 100% Ring: {sol_rows}"
    );
    let (_, vault) = app.get(&search("lim-dul"), Some(&token)).await;
    assert_eq!(vault["total"], 1, "accents fold: {vault}");
    let (_, percent) = app.get(&search("%25"), Some(&token)).await;
    assert_eq!(
        rows(&percent).len(),
        1,
        "% is text, not a wildcard: {percent}"
    );
    let (_, underscore) = app.get(&search("_"), Some(&token)).await;
    assert_eq!(underscore["total"], 0, "_ is text too: {underscore}");
    let (status, hostile) = app
        .get(
            &search("%27%3B%20DROP%20TABLE%20collection_cards%3B%20--"),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{hostile}");
    assert_eq!(hostile["total"], 0);
    assert_eq!(
        list(&app, &token).await["total"],
        5,
        "the table is as it was"
    );

    for query in [
        format!("name={}&offset=0&limit=10", "a".repeat(101)),
        "offset=0&limit=0".to_owned(),
        "offset=0&limit=101".to_owned(),
        "offset=-1&limit=10".to_owned(),
    ] {
        let (status, body) = app
            .get(&format!("{COLLECTION_ROUTE}?{query}"), Some(&token))
            .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{query}: {body}");
    }
    let (status, plain) = app.get(COLLECTION_ROUTE, Some(&token)).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "no query: the first page of 50: {plain}"
    );
    assert_eq!(plain["total"], 5);
}
