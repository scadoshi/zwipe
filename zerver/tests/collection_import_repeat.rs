//! A collection import sent again with its `request_id`, as a client does when the first reply
//! was lost, answers the first import's result and writes nothing. The id is kept a day.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![expect(
    clippy::unwrap_used,
    reason = "a test: helpers outside #[test] fns unwrap, and a panic is the failure report"
)]

mod common;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::atomic::{AtomicU8, Ordering};

use axum::http::{Method, StatusCode};
use common::{CardFixture, TestApp, card, seed_cards};
use serde_json::{Value, json};
use uuid::Uuid;
use zwipe_core::http::paths::IMPORT_COLLECTION_ROUTE;

const ARCHIDEKT_HEADER: &str = "Quantity,Name,Finish,Condition,Date Added,Language,Purchase Price,Tags,Edition Name,Edition Code,Multiverse Id,Scryfall ID,Collector Number";

/// Each import's own address, so the per-address limit only bites where a test means it to.
static PEER: AtomicU8 = AtomicU8::new(1);

fn peer() -> SocketAddr {
    let n = PEER.fetch_add(1, Ordering::Relaxed);
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(13, 0, 0, n.max(1))), 41000)
}

async fn sol_ring(pool: &sqlx::PgPool) -> CardFixture {
    let sol = card("Sol Ring")
        .set("c21", "Commander 2021")
        .collector_number("263");
    seed_cards(pool, std::slice::from_ref(&sol)).await;
    sol
}

/// An Archidekt export of `copies` Sol Rings.
fn export(sol: &CardFixture, copies: u32) -> String {
    format!(
        "{ARCHIDEKT_HEADER}\n{copies},Sol Ring,Normal,NM,2025-03-03,EN,,,Commander 2021,c21,,{},263\n",
        sol.id()
    )
}

async fn import(
    app: &TestApp,
    token: &str,
    csv: &str,
    mode: &str,
    request_id: Option<Uuid>,
) -> (StatusCode, Value) {
    let body = match request_id {
        Some(id) => json!({ "csv": csv, "mode": mode, "request_id": id }),
        None => json!({ "csv": csv, "mode": mode }),
    };
    app.send_from(
        peer(),
        Method::POST,
        IMPORT_COLLECTION_ROUTE,
        Some(body),
        Some(token),
    )
    .await
}

async fn copies(pool: &sqlx::PgPool, user_id: &str) -> i64 {
    sqlx::query_scalar(
        "SELECT COALESCE(sum(quantity), 0)::bigint FROM collection_cards WHERE user_id = $1",
    )
    .bind(user_id.parse::<Uuid>().unwrap())
    .fetch_one(pool)
    .await
    .unwrap()
}

#[sqlx::test]
async fn an_add_sent_again_with_its_request_id_answers_the_first_result_and_adds_nothing(
    pool: sqlx::PgPool,
) {
    let app = TestApp::new(pool.clone());
    let sol = sol_ring(&pool).await;
    let (token, uid) = app.register("repeater").await;
    let file = export(&sol, 4);
    let id = Uuid::new_v4();

    let (status, first) = import(&app, &token, &file, "add", Some(id)).await;
    assert_eq!(status, StatusCode::OK, "{first}");
    let (status, again) = import(&app, &token, &file, "add", Some(id)).await;
    assert_eq!(status, StatusCode::OK, "{again}");

    assert_eq!(again, first, "the repeat answers the first import's result");
    assert_eq!(copies(&pool, &uid).await, 4, "the repeat added nothing");
}

#[sqlx::test]
async fn an_add_without_a_request_id_sums_every_time(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let sol = sol_ring(&pool).await;
    let (token, uid) = app.register("oldclient").await;
    let file = export(&sol, 4);

    for _ in 0..2 {
        let (status, body) = import(&app, &token, &file, "add", None).await;
        assert_eq!(status, StatusCode::OK, "{body}");
    }
    assert_eq!(
        copies(&pool, &uid).await,
        8,
        "an old client's import is unchanged"
    );
}

#[sqlx::test]
async fn a_new_request_id_imports_again(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let sol = sol_ring(&pool).await;
    let (token, uid) = app.register("twopicks").await;
    let file = export(&sol, 4);

    for _ in 0..2 {
        let (status, body) = import(&app, &token, &file, "add", Some(Uuid::new_v4())).await;
        assert_eq!(status, StatusCode::OK, "{body}");
    }
    assert_eq!(copies(&pool, &uid).await, 8, "two picks are two imports");
}

#[sqlx::test]
async fn a_request_id_sent_with_another_file_is_turned_down_and_changes_nothing(
    pool: sqlx::PgPool,
) {
    let app = TestApp::new(pool.clone());
    let sol = sol_ring(&pool).await;
    let (token, uid) = app.register("reuser").await;
    let id = Uuid::new_v4();

    let (status, body) = import(&app, &token, &export(&sol, 4), "add", Some(id)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    for (csv, mode) in [(export(&sol, 5), "add"), (export(&sol, 4), "replace")] {
        let (status, body) = import(&app, &token, &csv, mode, Some(id)).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{mode}: {body}");
    }
    assert_eq!(copies(&pool, &uid).await, 4);
}

#[sqlx::test]
async fn a_request_id_past_a_day_old_imports_again(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let sol = sol_ring(&pool).await;
    let (token, uid) = app.register("tomorrow").await;
    let file = export(&sol, 4);
    let id = Uuid::new_v4();

    let (status, body) = import(&app, &token, &file, "add", Some(id)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    sqlx::query(
        "UPDATE collection_import_requests SET created_at = now() - interval '25 hours' \
         WHERE request_id = $1",
    )
    .bind(id)
    .execute(&pool)
    .await
    .unwrap();
    let (status, body) = import(&app, &token, &file, "add", Some(id)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(copies(&pool, &uid).await, 8);
}

#[sqlx::test]
async fn one_user_s_request_id_is_nothing_to_another(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let sol = sol_ring(&pool).await;
    let (first_token, _) = app.register("firstuser").await;
    let (second_token, second_uid) = app.register("seconduser").await;
    let file = export(&sol, 4);
    let id = Uuid::new_v4();

    for token in [&first_token, &second_token] {
        let (status, body) = import(&app, token, &file, "add", Some(id)).await;
        assert_eq!(status, StatusCode::OK, "{body}");
    }
    assert_eq!(copies(&pool, &second_uid).await, 4);
}
