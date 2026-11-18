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

use std::net::SocketAddr;

use axum::http::StatusCode;
use common::{CardFixture, TestApp, card, seed_cards};
use serde_json::{Value, json};
use uuid::Uuid;
use zwipe::domain::card::models::zervice_metrics::ZerviceMetrics;
use zwipe::domain::card::ports::CardRepository;
use zwipe::domain::card::requests::get_scryfall_data::GetScryfallData;
use zwipe::outbound::sqlx::postgres::Postgres;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::collection::{CollectionPageSize, CollectionSearch, Finish, SetQuantity};
use zwipe_core::http::contracts::collection::HttpSetCollectionCard;
use zwipe_core::http::endpoints::collection::{GetCollection, SetCollectionCard};
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
        .get(&format!("{COLLECTION_ROUTE}?limit=100"), Some(token))
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

/// Every page of the caller's list for the search `name`, 2 rows a page, following each reply's
/// `next` until there is none.
async fn walk(app: &TestApp, token: &str, name: &str) -> Vec<Value> {
    let mut pages = Vec::new();
    let mut after: Option<String> = None;
    loop {
        let mut parts = vec!["limit=2".to_owned()];
        if !name.is_empty() {
            parts.push(format!("name={name}"));
        }
        if let Some(cursor) = &after {
            parts.push(format!("after={cursor}"));
        }
        let query = format!("{COLLECTION_ROUTE}?{}", parts.join("&"));
        let (status, page) = app.get(&query, Some(token)).await;
        assert_eq!(status, StatusCode::OK, "{query}: {page}");
        after = page["next"].as_str().map(str::to_owned);
        pages.push(page);
        if after.is_none() || pages.len() > 20 {
            return pages;
        }
    }
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

    let pages = walk(&app, &token, "").await;
    assert_eq!(pages.len(), 3, "5 rows, 2 a page");
    for page in &pages {
        assert_eq!(page["total"], 5);
    }
    assert_eq!(pages[2]["next"], Value::Null, "the last page");
    let [first, second, last] = [pages[0].clone(), pages[1].clone(), pages[2].clone()];
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

    let search = |name: &str| format!("{COLLECTION_ROUTE}?name={name}&limit=100");
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
        format!("name={}&limit=10", "a".repeat(101)),
        "limit=0".to_owned(),
        "limit=101".to_owned(),
        "after=ZZ&limit=10".to_owned(),
        "after=0a1b&limit=10".to_owned(),
        format!("after={}&limit=10", hex::encode("[\"x\"]")),
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

/// The list and the set call made through `zwipe-client`, against the router served on a local
/// port, so the URL each is sent to is the one the client builds: the search, with `'`, `&` and
/// a space in it, has to reach the route as its query.
#[sqlx::test]
async fn the_client_s_calls_reach_the_routes(pool: sqlx::PgPool) {
    let vault = card("Lim-Dûl's Vault & Co");
    let bolt = card("Lightning Bolt").mono("R");
    seed_cards(&pool, &[vault.clone(), bolt.clone()]).await;
    let app = TestApp::new(pool);
    let session: Session =
        serde_json::from_value(app.register_on("clientcall", None).await).unwrap();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let service = app
        .router
        .clone()
        .into_make_service_with_connect_info::<SocketAddr>();
    tokio::spawn(async move { axum::serve(listener, service).await });
    let client = ZwipeClient::new(format!("http://{address}").parse().unwrap());

    for printing in [&vault, &bolt] {
        let call = HttpSetCollectionCard {
            scryfall_data_id: printing.id(),
            finish: Finish::Foil,
            condition: None,
            language: None,
            quantity: SetQuantity::new(2).unwrap(),
        };
        let reply = client
            .call(SetCollectionCard(call), Some(&session))
            .await
            .unwrap();
        assert!(reply.card.is_some(), "stored");
    }

    let page = client
        .call(
            GetCollection {
                name: CollectionSearch::new("dûl's vault & co").unwrap(),
                after: None,
                limit: CollectionPageSize::new(1).unwrap(),
            },
            Some(&session),
        )
        .await
        .unwrap();
    assert_eq!(page.total, 1, "the search reached the route as its query");
    let names: Vec<&str> = page
        .cards
        .iter()
        .map(|row| row.card.scryfall_data.name.as_str())
        .collect();
    assert_eq!(names, ["Lim-Dûl's Vault & Co"]);

    let first = client
        .call(
            GetCollection {
                name: CollectionSearch::default(),
                after: None,
                limit: CollectionPageSize::new(1).unwrap(),
            },
            Some(&session),
        )
        .await
        .unwrap();
    assert!(first.next.is_some(), "1 of 2 with 1 a page has a next");
    let every = client
        .call(
            GetCollection {
                name: CollectionSearch::default(),
                after: first.next,
                limit: CollectionPageSize::new(1).unwrap(),
            },
            Some(&session),
        )
        .await
        .unwrap();
    assert_eq!(every.total, 2);
    assert_eq!(every.next, None, "the second of 2 is the last");
    assert_eq!(
        every
            .cards
            .first()
            .map(|row| row.card.scryfall_data.name.as_str()),
        Some("Lim-Dûl's Vault & Co"),
        "the cursor reached the route too"
    );
}

/// A row added before the cursor between two pages moves nothing after it: the next page starts
/// right after the last row sent, with no row repeated or skipped.
#[sqlx::test]
async fn a_page_after_a_cursor_holds_whatever_lands_before_it(pool: sqlx::PgPool) {
    let printings: Vec<CardFixture> = ["Bolt", "Counterspell", "Dark Ritual", "Elves"]
        .iter()
        .map(|name| card(name))
        .collect();
    let early = card("Ancestral Recall");
    seed_cards(&pool, &[printings.clone(), vec![early.clone()]].concat()).await;
    let app = TestApp::new(pool.clone());
    let (token, user) = app.register("cursor").await;
    for printing in &printings {
        imported(&pool, &user, printing, "nonfoil", None, None, 1).await;
    }

    let (_, first) = app
        .get(&format!("{COLLECTION_ROUTE}?limit=2"), Some(&token))
        .await;
    let first_names: Vec<String> = rows(&first).into_iter().map(|row| row.0).collect();
    assert_eq!(first_names, ["Bolt", "Counterspell"]);
    imported(&pool, &user, &early, "nonfoil", None, None, 1).await;
    let after = first["next"].as_str().unwrap();
    let (_, second) = app
        .get(
            &format!("{COLLECTION_ROUTE}?after={after}&limit=2"),
            Some(&token),
        )
        .await;
    let second_names: Vec<String> = rows(&second).into_iter().map(|row| row.0).collect();
    assert_eq!(second_names, ["Dark Ritual", "Elves"], "no repeat, no skip");
}

/// A printing the nightly sync renames moves in the list and answers to its new name.
#[sqlx::test]
async fn a_renamed_printing_lists_and_searches_under_its_new_name(pool: sqlx::PgPool) {
    let old = card("Llanowar Elves");
    seed_cards(&pool, std::slice::from_ref(&old)).await;
    let app = TestApp::new(pool.clone());
    let (token, user) = app.register("renamed").await;
    imported(&pool, &user, &old, "nonfoil", None, None, 1).await;

    let repo = Postgres::from_pool(pool.clone());
    let mut renamed = repo
        .fetch_scryfall_data(&GetScryfallData::new(&old.id().to_string()).unwrap())
        .await
        .unwrap();
    renamed.name = "Llanowar Lurkers".to_owned();
    let mut metrics = ZerviceMetrics::new();
    let _ = repo
        .batch_delta_upsert(std::slice::from_ref(&renamed), 100, &mut metrics)
        .await
        .unwrap();

    let (_, found) = app
        .get(
            &format!("{COLLECTION_ROUTE}?name=lurkers&limit=10"),
            Some(&token),
        )
        .await;
    assert_eq!(found["total"], 1, "the new name finds it: {found}");
    let (_, gone) = app
        .get(
            &format!("{COLLECTION_ROUTE}?name=elves&limit=10"),
            Some(&token),
        )
        .await;
    assert_eq!(gone["total"], 0, "the old name doesn't: {gone}");
}
