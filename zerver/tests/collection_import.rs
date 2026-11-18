//! Collection import against a real database: a ManaBox and an Archidekt export land, an add
//! sums into what's there and a replace removes what the export doesn't hold, and every hostile
//! file is either read as plain data or turned down whole, leaving the collection as it was.
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
use zwipe_core::domain::collection::limits::{MAX_BODY_BYTES, MAX_CELL_CHARS, MAX_ROWS};
use zwipe_core::http::paths::IMPORT_COLLECTION_ROUTE;

const MANABOX_HEADER: &str = "Name,Set code,Set name,Collector number,Foil,Rarity,Quantity,ManaBox ID,Scryfall ID,Purchase price,Misprint,Altered,Signed,Condition,Language,Proxy,Purchase price currency,Added";
const ARCHIDEKT_HEADER: &str = "Quantity,Name,Finish,Condition,Date Added,Language,Purchase Price,Tags,Edition Name,Edition Code,Multiverse Id,Scryfall ID,Collector Number";

/// Each import's own address, so the per-address limit only bites where a test means it to.
static PEER: AtomicU8 = AtomicU8::new(1);

fn peer() -> SocketAddr {
    let n = PEER.fetch_add(1, Ordering::Relaxed);
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(12, 0, 0, n.max(1))), 41000)
}

struct Cards {
    sol: CardFixture,
    bolt: CardFixture,
    forest_a: CardFixture,
    forest_b: CardFixture,
}

async fn seeded(pool: &sqlx::PgPool) -> Cards {
    let cards = Cards {
        sol: card("Sol Ring")
            .set("c21", "Commander 2021")
            .collector_number("263"),
        bolt: card("Lightning Bolt")
            .set("m10", "Magic 2010")
            .collector_number("146"),
        forest_a: card("Forest")
            .set("m10", "Magic 2010")
            .collector_number("246"),
        forest_b: card("Forest")
            .set("c21", "Commander 2021")
            .collector_number("400"),
    };
    seed_cards(
        pool,
        &[
            cards.sol.clone(),
            cards.bolt.clone(),
            cards.forest_a.clone(),
            cards.forest_b.clone(),
        ],
    )
    .await;
    cards
}

async fn import(app: &TestApp, token: Option<&str>, csv: &str, mode: &str) -> (StatusCode, Value) {
    app.send_from(
        peer(),
        Method::POST,
        IMPORT_COLLECTION_ROUTE,
        Some(json!({ "csv": csv, "mode": mode })),
        token,
    )
    .await
}

/// A collection row as `(printing, finish, condition, language, copies, tags, price, currency,
/// added on)`.
type Row = (
    Uuid,
    String,
    Option<String>,
    Option<String>,
    i32,
    Vec<String>,
    Option<String>,
    Option<String>,
    Option<String>,
);

async fn rows(pool: &sqlx::PgPool, user_id: &str) -> Vec<Row> {
    sqlx::query_as(
        "SELECT scryfall_data_id, finish, condition, language, quantity, tags, purchase_price::text, \
             purchase_currency, to_char(added_at AT TIME ZONE 'UTC', 'YYYY-MM-DD') \
         FROM collection_cards WHERE user_id = $1 ORDER BY scryfall_data_id, finish",
    )
    .bind(user_id.parse::<Uuid>().unwrap())
    .fetch_all(pool)
    .await
    .unwrap()
}

/// Rows in every table an import could reach.
async fn tables(pool: &sqlx::PgPool) -> (i64, i64, i64) {
    sqlx::query_as(
        "SELECT (SELECT count(*) FROM collection_cards), (SELECT count(*) FROM users), \
             (SELECT count(*) FROM scryfall_data)",
    )
    .fetch_one(pool)
    .await
    .unwrap()
}

fn tags(list: &[&str]) -> Vec<String> {
    list.iter().map(ToString::to_string).collect()
}

#[sqlx::test]
async fn a_manabox_export_lands_and_says_what_didnt(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let c = seeded(&pool).await;
    let (token, uid) = app.register("boxer").await;
    let unknown = Uuid::new_v4();
    let csv = format!(
        "{MANABOX_HEADER}
Sol Ring,C21,Commander 2021,263,normal,uncommon,2,1,{sol},0.41,false,false,false,near_mint,en,false,USD,2026-08-01T15:31:19.629Z
Lightning Bolt,M10,Magic 2010,146,foil,common,1,2,{unknown},,false,false,false,near_mint,en,false,USD,2026-08-02T10:00:00.000Z
Forest,ZZZ,Nowhere,999,normal,common,4,3,,,false,false,false,near_mint,en,false,USD,
Sol Ring,C21,Commander 2021,263,normal,uncommon,1,1,{sol},,false,false,false,near_mint,en,false,USD,2026-07-01T00:00:00.000Z
",
        sol = c.sol.id()
    );

    let (status, summary) = import(&app, Some(&token), &csv, "add").await;

    assert_eq!(status, StatusCode::OK, "{summary}");
    assert_eq!(
        summary,
        json!({
            "source": "mana_box", "rows": 4, "added": 2, "updated": 0, "removed": 0, "cards": 4,
            "unmatched": [{ "line": 4, "name": "Forest", "set": "zzz", "reason": "ambiguous_name" }],
            "unmatched_total": 1,
        })
    );
    let mut want = vec![
        (
            c.sol.id(),
            "nonfoil".into(),
            Some("near_mint".into()),
            Some("en".into()),
            3,
            vec![],
            Some("0.41".into()),
            Some("USD".into()),
            Some("2026-07-01".into()),
        ),
        (
            c.bolt.id(),
            "foil".into(),
            Some("near_mint".into()),
            Some("en".into()),
            1,
            vec![],
            None,
            None,
            Some("2026-08-02".into()),
        ),
    ];
    want.sort();
    assert_eq!(
        rows(&pool, &uid).await,
        want,
        "the two Sol Ring rows merged; Bolt matched by set and number"
    );
}

#[sqlx::test]
async fn an_add_sums_into_the_collection_and_a_replace_makes_it_the_export(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let c = seeded(&pool).await;
    let (token, uid) = app.register("dekt").await;
    let sol = |copies: u32, day: &str, price: &str, cell: &str| {
        format!(
            "{copies},Sol Ring,Normal,NM,{day},EN,{price},\"{cell}\",Commander 2021,c21,,{},263",
            c.sol.id()
        )
    };
    let bolt = |copies: u32| {
        format!(
            "{copies},Lightning Bolt,Foil,LP,2025-03-03,EN,,,Magic 2010,m10,,{},146",
            c.bolt.id()
        )
    };

    let first = format!(
        "{ARCHIDEKT_HEADER}\n{}\n",
        sol(1, "2025-03-03", "1.50", "trade, Binder")
    );
    let (status, summary) = import(&app, Some(&token), &first, "add").await;
    assert_eq!(status, StatusCode::OK, "{summary}");
    assert_eq!(
        (summary["source"].clone(), summary["added"].clone()),
        (json!("archidekt"), json!(1))
    );

    let second = format!(
        "{ARCHIDEKT_HEADER}\n{}\n{}\n",
        sol(2, "2025-03-01", "", "TRADE, deck"),
        bolt(1)
    );
    let (status, summary) = import(&app, Some(&token), &second, "add").await;
    assert_eq!(status, StatusCode::OK, "{summary}");
    assert_eq!(
        (
            summary["added"].clone(),
            summary["updated"].clone(),
            summary["cards"].clone()
        ),
        (json!(1), json!(1), json!(4))
    );
    let mut want = vec![
        (
            c.sol.id(),
            "nonfoil".into(),
            Some("near_mint".into()),
            Some("en".into()),
            3,
            tags(&["trade", "Binder", "deck"]),
            Some("1.50".into()),
            None,
            Some("2025-03-01".into()),
        ),
        (
            c.bolt.id(),
            "foil".into(),
            Some("lightly_played".into()),
            Some("en".into()),
            1,
            vec![],
            None,
            None,
            Some("2025-03-03".into()),
        ),
    ];
    want.sort();
    assert_eq!(
        rows(&pool, &uid).await,
        want,
        "copies summed, tags joined without regard to case, the earlier day and the price kept"
    );

    let third = format!("{ARCHIDEKT_HEADER}\n{}\n", bolt(4));
    let (status, summary) = import(&app, Some(&token), &third, "replace").await;
    assert_eq!(status, StatusCode::OK, "{summary}");
    assert_eq!(
        (
            summary["added"].clone(),
            summary["updated"].clone(),
            summary["removed"].clone(),
            summary["cards"].clone()
        ),
        (json!(0), json!(1), json!(1), json!(4))
    );
    assert_eq!(
        rows(&pool, &uid).await,
        vec![(
            c.bolt.id(),
            "foil".into(),
            Some("lightly_played".into()),
            Some("en".into()),
            4,
            vec![],
            None,
            None,
            Some("2025-03-03".into())
        )],
        "the replace set Bolt to the export's 4 and removed Sol Ring"
    );
}

/// A user holding one Sol Ring, for a hostile import to try to change.
async fn holder(app: &TestApp, c: &Cards, name: &str) -> (String, String) {
    let (token, uid) = app.register(name).await;
    let csv = format!(
        "{ARCHIDEKT_HEADER}\n1,Sol Ring,Normal,NM,2025-03-03,EN,,,Commander 2021,c21,,{},263\n",
        c.sol.id()
    );
    let (status, body) = import(app, Some(&token), &csv, "add").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    (token, uid)
}

#[sqlx::test]
async fn a_file_it_cant_take_changes_nothing(pool: sqlx::PgPool) {
    let c = seeded(&pool).await;
    let long_cell = format!(
        "{ARCHIDEKT_HEADER}\n1,{},Normal,NM,,EN,,,x,c21,,,263\n",
        "x".repeat(MAX_CELL_CHARS + 1)
    );
    let mut too_many_rows = format!("{ARCHIDEKT_HEADER}\n");
    for _ in 0..=MAX_ROWS {
        too_many_rows.push_str("1,a,,,,,,,,,,,\n");
    }
    let too_many_tags = format!(
        "{ARCHIDEKT_HEADER}\n1,Sol Ring,Normal,NM,,EN,,\"{}\",x,c21,,,263\n",
        (0..=20)
            .map(|n| format!("t{n}"))
            .collect::<Vec<_>>()
            .join(",")
    );
    let junk: String = (0..4_000u32)
        .map(|n| char::from_u32(n % 0xd7ff).unwrap_or('\u{fffd}'))
        .collect();
    let nothing_matches = format!("{ARCHIDEKT_HEADER}\n1,No Such Card,Normal,NM,,EN,,,x,zzz,,,1\n");
    let cases: [(&str, String, &str); 8] = [
        ("an empty file", String::new(), "add"),
        (
            "a header and no rows",
            format!("{ARCHIDEKT_HEADER}\n"),
            "add",
        ),
        ("a cell past the cap", long_cell, "add"),
        ("more rows than the cap", too_many_rows, "add"),
        ("more tags than the cap", too_many_tags, "add"),
        ("binary junk", junk, "add"),
        (
            "a header neither app writes",
            "Card,Count\nSol Ring,1\n".into(),
            "add",
        ),
        (
            "a replace where nothing matches",
            nothing_matches,
            "replace",
        ),
    ];
    for (n, (what, csv, mode)) in cases.into_iter().enumerate() {
        // Its own app, so the sign-ups stay under the registration limit.
        let app = TestApp::new(pool.clone());
        let (token, uid) = holder(&app, &c, &format!("hostile{n}")).await;
        let before = (rows(&pool, &uid).await, tables(&pool).await);
        let (status, _) = import(&app, Some(&token), &csv, mode).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{what}");
        assert_eq!(
            (rows(&pool, &uid).await, tables(&pool).await),
            before,
            "{what} changed nothing"
        );
    }
}

#[sqlx::test]
async fn sql_and_formula_text_is_only_ever_data(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let c = seeded(&pool).await;
    let (token, uid) = app.register("bobby").await;
    let before = tables(&pool).await;
    let csv = format!(
        "{ARCHIDEKT_HEADER}
1,\"x'); DROP TABLE collection_cards; --\",Normal,NM,,EN,,,x,zzz,,,1
1,=HYPERLINK(\"http://evil\"),Normal,NM,,EN,,,x,zzz,,,2
1,Sol Ring,Normal,NM,,EN,\"=1+1\",\"'; DELETE FROM users; --, =SUM(A1:A9), @cmd\",Commander 2021,c21,,{},263
",
        c.sol.id()
    );

    let (status, summary) = import(&app, Some(&token), &csv, "add").await;

    assert_eq!(status, StatusCode::OK, "{summary}");
    assert_eq!(summary["unmatched_total"], json!(2), "{summary}");
    assert_eq!(
        summary["unmatched"][0]["name"],
        json!("x'); DROP TABLE collection_cards; --")
    );
    let held = rows(&pool, &uid).await;
    assert_eq!(held.len(), 1);
    assert_eq!(
        held[0].5,
        tags(&["'; DELETE FROM users; --", "=SUM(A1:A9)", "@cmd"]),
        "stored as written"
    );
    assert_eq!(held[0].6, None, "a formula is not a price");
    let after = tables(&pool).await;
    assert_eq!(
        (after.1, after.2),
        (before.1, before.2),
        "users and cards untouched"
    );
}

#[sqlx::test]
async fn a_utf16_file_read_as_text_imports_what_it_says(pool: sqlx::PgPool) {
    // A browser reading a UTF-16 file as UTF-8 hands over its byte order mark as replacement
    // characters and a NUL after every letter; the reader drops the NULs.
    let app = TestApp::new(pool.clone());
    let c = seeded(&pool).await;
    let (token, uid) = app.register("utf16").await;
    let text = format!(
        "{ARCHIDEKT_HEADER}\n2,Sol Ring,Normal,NM,,EN,,,x,c21,,{},263\n",
        c.sol.id()
    );
    let utf16: String = std::iter::once("\u{fffd}\u{fffd}".to_owned())
        .chain(text.chars().map(|ch| format!("{ch}\u{0}")))
        .collect();

    let (status, summary) = import(&app, Some(&token), &utf16, "add").await;

    assert_eq!(status, StatusCode::OK, "{summary}");
    assert_eq!(
        rows(&pool, &uid)
            .await
            .iter()
            .map(|r| (r.0, r.4))
            .collect::<Vec<_>>(),
        vec![(c.sol.id(), 2)]
    );
}

/// `csv` padded to about `bytes` with rows of long names that match nothing.
fn sized(csv: &str, bytes: usize) -> String {
    let filler = format!("1,{},,,,,,,,,,,\n", "x".repeat(MAX_CELL_CHARS - 12));
    let mut out = csv.to_owned();
    while out.len() < bytes {
        out.push_str(&filler);
    }
    out
}

#[sqlx::test]
async fn the_route_holds_its_caps(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let c = seeded(&pool).await;
    let (token, uid) = app.register("capped").await;
    let one = format!(
        "{ARCHIDEKT_HEADER}\n1,Sol Ring,Normal,NM,,EN,,,x,c21,,{},263\n",
        c.sol.id()
    );

    let (status, _) = import(&app, None, &one, "add").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "no session");

    // Another account for the turned-down bodies, which spend its budget too.
    let (other, _) = app.register("capped2").await;
    let over = sized(&one, MAX_BODY_BYTES / 8 * 9);
    let (status, _) = import(&app, Some(&other), &over, "add").await;
    assert_eq!(
        status,
        StatusCode::PAYLOAD_TOO_LARGE,
        "a 4.5 MiB body is past the cap"
    );

    let (status, _) = app
        .send_from(
            peer(),
            Method::POST,
            IMPORT_COLLECTION_ROUTE,
            Some(json!({ "csv": one })),
            Some(&other),
        )
        .await;
    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "the mode is required"
    );

    let (status, _) = import(&app, Some(&token), &one, "add").await;
    assert_eq!(status, StatusCode::OK);
    // Past the 2 MiB the other routes take, under the import's own cap.
    let big = sized(&one, MAX_BODY_BYTES / 8 * 7);
    assert!(big.len() > 2 * 1024 * 1024);
    let (status, summary) = import(&app, Some(&token), &big, "add").await;
    assert_eq!(
        status,
        StatusCode::OK,
        "a 3.5 MiB file gets through: {summary}"
    );
    let (status, _) = import(&app, Some(&token), &one, "add").await;
    assert_eq!(status, StatusCode::OK, "the third of the burst");
    let (status, _) = import(&app, Some(&token), &one, "add").await;
    assert_eq!(
        status,
        StatusCode::TOO_MANY_REQUESTS,
        "past the account's budget, from a new address"
    );
    let copies: Vec<i32> = rows(&pool, &uid).await.iter().map(|r| r.4).collect();
    assert_eq!(copies, vec![3], "the three imports that got through landed");
}

/// An import serializes with the user's other imports, not with the rest of their writes: it
/// finishes while another transaction is writing to the user's row.
#[sqlx::test]
async fn an_import_doesnt_wait_on_the_user_s_row(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let c = seeded(&pool).await;
    let (token, uid) = app.register("unblocked").await;
    let mut other = pool.begin().await.unwrap();
    // An ordinary write to the user's row, as a settings change makes.
    sqlx::query("UPDATE users SET updated_at = updated_at WHERE id = $1")
        .bind(uid.parse::<Uuid>().unwrap())
        .execute(&mut *other)
        .await
        .unwrap();
    let one = format!(
        "{ARCHIDEKT_HEADER}\n1,Sol Ring,Normal,NM,,EN,,,x,c21,,{},263\n",
        c.sol.id()
    );

    let done = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        import(&app, Some(&token), &one, "add"),
    )
    .await;

    other.rollback().await.unwrap();
    let (status, body) = done.expect("the import finished while the user's row was being written");
    assert_eq!(status, StatusCode::OK, "{body}");
}

/// A name alone matches only when one printing has it; a name within a set narrows it to that
/// set's printing.
#[sqlx::test]
async fn a_name_matches_within_its_set_or_alone_when_unique(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let c = seeded(&pool).await;
    let (token, uid) = app.register("byname").await;
    let csv = format!(
        "{ARCHIDEKT_HEADER}
1,Forest,Normal,NM,,EN,,,x,m10,,,
2,Lightning Bolt,Normal,NM,,EN,,,x,,,,
1,Forest,Normal,NM,,EN,,,x,,,,
"
    );

    let (status, summary) = import(&app, Some(&token), &csv, "add").await;

    assert_eq!(status, StatusCode::OK, "{summary}");
    let held: Vec<(Uuid, i32)> = rows(&pool, &uid).await.iter().map(|r| (r.0, r.4)).collect();
    let mut want = vec![(c.forest_a.id(), 1), (c.bolt.id(), 2)];
    want.sort();
    assert_eq!(
        held, want,
        "Forest in m10 is that printing; Bolt has one printing"
    );
    assert_eq!(
        summary["unmatched"][0]["reason"],
        json!("ambiguous_name"),
        "Forest alone is two printings"
    );
}
