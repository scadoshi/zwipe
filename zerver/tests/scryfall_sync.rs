//! zervice's Scryfall syncs against fixture files: a good file loads as it always has, and an
//! empty, truncated or corrupt file, a broken or incomplete line, a far smaller set, a mass
//! removal or an older file fails its step and leaves the stored rows as they were.
//!
//! The fixtures are text under `tests/fixtures/scryfall/`, gzipped here, and reach the syncs
//! through [`Fixture`], a [`BulkSource`] that serves them in place of Scryfall.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "a test: helpers outside #[test] fns unwrap and index the fixture's JSON, and a panic is the failure report"
)]

mod common;

use std::collections::HashSet;
use std::future::{self, Future};
use std::io::Write;

use chrono::{DateTime, TimeZone, Utc};
use flate2::Compression;
use flate2::write::GzEncoder;
use serde_json::Value;
use uuid::Uuid;
use zwipe::domain::card;
use zwipe::domain::card::models::printing_removal::RemovalMode;
use zwipe::domain::card::models::scryfall_pull::{RefusedPull, ScryfallLoad};
use zwipe::domain::card::ports::CardService;
use zwipe::inbound::external::scryfall::bulk::{self, BulkEndpoint, BulkFile, BulkSource};
use zwipe::inbound::external::scryfall::oracle_tag::OracleTag;
use zwipe::outbound::sqlx::card::helpers::derive_categories::CATEGORY_ROOTS;
use zwipe::outbound::sqlx::card::helpers::scryfall_loads;
use zwipe::outbound::sqlx::postgres::Postgres;

/// One real printing, the template every card line is made from.
const CARD: &str = include_str!("fixtures/scryfall/card.json");

/// The oracle tags fixture: three tags on three cards, a blank line and a tagging without a card.
const ORACLE_TAGS: &str = include_str!("fixtures/scryfall/oracle_tags.jsonl");

/// Gzips `text` as Scryfall serves a bulk file.
fn gz(text: &str) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(text.as_bytes()).unwrap();
    encoder.finish().unwrap()
}

/// Noon UTC on 2026-11-`day`.
fn day(day: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 11, day, 12, 0, 0).unwrap()
}

/// One bulk file served in place of Scryfall: its listing and its bytes.
#[derive(Debug, Clone)]
struct Fixture {
    file: BulkFile,
    bytes: Vec<u8>,
}

impl Fixture {
    /// `kind`'s file holding `text`, made on `made`, listed at its true size.
    fn new(kind: BulkEndpoint, text: &str, made: DateTime<Utc>) -> Self {
        let bytes = gz(text);
        Self {
            file: BulkFile {
                kind: kind.kind(),
                uri: format!("https://data.scryfall.io/{}/fixture.jsonl.gz", kind.kind()),
                updated_at: Some(made),
                compressed_size: Some(u64::try_from(bytes.len()).unwrap()),
            },
            bytes,
        }
    }

    /// The same listing over `bytes`, as a download cut short or mangled serves it.
    fn serving(mut self, bytes: Vec<u8>) -> Self {
        self.bytes = bytes;
        self
    }
}

impl BulkSource for Fixture {
    fn describe(
        &self,
        _endpoint: BulkEndpoint,
    ) -> impl Future<Output = anyhow::Result<BulkFile>> + Send {
        future::ready(Ok(self.file.clone()))
    }

    fn download(&self, _uri: &str) -> impl Future<Output = anyhow::Result<Vec<u8>>> + Send {
        future::ready(Ok(self.bytes.clone()))
    }
}

/// The refusal inside `error`, if it is one.
fn refusal(error: &anyhow::Error) -> Option<&RefusedPull> {
    error.downcast_ref::<RefusedPull>()
}

// == the pull ==

#[tokio::test]
async fn a_good_file_pulls_every_line_and_skips_blanks() {
    let fixture = Fixture::new(BulkEndpoint::OracleTags, ORACLE_TAGS, day(19));
    let pulled = bulk::pull::<OracleTag, _>(&fixture, fixture.file.clone())
        .await
        .unwrap();
    let slugs: Vec<&str> = pulled.items.iter().map(|t| t.slug.as_str()).collect();
    assert_eq!(slugs, ["removal", "ramp", "draw"]);
}

#[tokio::test]
async fn an_empty_file_pulls_nothing_and_leaves_the_refusal_to_its_sync() {
    let fixture = Fixture::new(BulkEndpoint::OracleTags, "", day(19));
    let pulled = bulk::pull::<OracleTag, _>(&fixture, fixture.file.clone())
        .await
        .unwrap();
    assert!(pulled.items.is_empty());
}

#[tokio::test]
async fn a_download_shorter_than_its_listing_is_refused_as_truncated() {
    let good = Fixture::new(BulkEndpoint::OracleTags, ORACLE_TAGS, day(19));
    let cut = good.bytes[..good.bytes.len() / 2].to_vec();
    let fixture = good.serving(cut);
    let error = bulk::pull::<OracleTag, _>(&fixture, fixture.file.clone())
        .await
        .unwrap_err();
    assert!(
        matches!(refusal(&error), Some(RefusedPull::Truncated { .. })),
        "{error:#}"
    );
}

#[tokio::test]
async fn a_truncated_gzip_with_no_size_listed_fails_to_parse() {
    let good = Fixture::new(BulkEndpoint::OracleTags, ORACLE_TAGS, day(19));
    let cut = good.bytes[..good.bytes.len() / 2].to_vec();
    let mut fixture = good.serving(cut);
    fixture.file.compressed_size = None;
    let error = bulk::pull::<OracleTag, _>(&fixture, fixture.file.clone())
        .await
        .unwrap_err();
    assert!(refusal(&error).is_none(), "{error:#}");
    assert!(format!("{error:#}").contains("jsonl"), "{error:#}");
}

#[tokio::test]
async fn a_corrupt_gzip_fails_to_parse() {
    let good = Fixture::new(BulkEndpoint::OracleTags, ORACLE_TAGS, day(19));
    let mut mangled = good.bytes.clone();
    for byte in mangled.iter_mut().skip(12).take(24) {
        *byte = !*byte;
    }
    let fixture = good.serving(mangled);
    assert!(
        bulk::pull::<OracleTag, _>(&fixture, fixture.file.clone())
            .await
            .is_err()
    );
}

#[tokio::test]
async fn a_broken_json_line_fails_the_whole_pull() {
    let text = format!("{ORACLE_TAGS}{{\"object\":\"tag\",\"slug\":\n");
    let fixture = Fixture::new(BulkEndpoint::OracleTags, &text, day(19));
    let error = bulk::pull::<OracleTag, _>(&fixture, fixture.file.clone())
        .await
        .unwrap_err();
    assert!(format!("{error:#}").contains("line 5"), "{error:#}");
}

#[tokio::test]
async fn a_line_missing_a_required_field_fails_the_whole_pull() {
    let text = format!(
        "{ORACLE_TAGS}{{\"object\":\"tag\",\"id\":\"00000000-0000-0000-0000-0000000000a4\",\"label\":\"No slug\"}}\n"
    );
    let fixture = Fixture::new(BulkEndpoint::OracleTags, &text, day(19));
    let error = bulk::pull::<OracleTag, _>(&fixture, fixture.file.clone())
        .await
        .unwrap_err();
    assert!(format!("{error:#}").contains("line 5"), "{error:#}");
}

// == the load record ==

#[sqlx::test]
async fn a_load_record_only_ever_rises(pool: sqlx::PgPool) {
    let mut conn = pool.acquire().await.unwrap();
    let load = |made: DateTime<Utc>, items: u64| ScryfallLoad {
        kind: "oracle_tags",
        updated_at: made,
        uri: format!("https://data.scryfall.io/oracle-tags/{items}.jsonl.gz"),
        items,
    };
    assert_eq!(
        scryfall_loads::fetch_load(&mut conn, "oracle_tags")
            .await
            .unwrap(),
        None
    );

    scryfall_loads::record_load(&mut conn, &load(day(19), 4_606))
        .await
        .unwrap();
    scryfall_loads::record_load(&mut conn, &load(day(18), 4_590))
        .await
        .unwrap();
    assert_eq!(
        scryfall_loads::fetch_load(&mut conn, "oracle_tags")
            .await
            .unwrap(),
        Some(day(19)),
        "an older file never lowers the mark"
    );

    scryfall_loads::record_load(&mut conn, &load(day(20), 4_610))
        .await
        .unwrap();
    let items: i64 =
        sqlx::query_scalar("SELECT items FROM scryfall_loads WHERE kind = 'oracle_tags'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(items, 4_610);
    assert_eq!(
        scryfall_loads::fetch_load(&mut conn, "default_cards")
            .await
            .unwrap(),
        None,
        "each kind keeps its own mark"
    );
}

// == the card sync ==

/// A card service over `pool`.
fn cards(pool: &sqlx::PgPool) -> card::services::Service<Postgres> {
    card::services::Service::new(Postgres::from_pool(pool.clone()))
}

/// One printing as a bulk line: printing `n` of card `oracle`, priced at `usd`.
fn card_line(n: u128, oracle: u128, usd: Option<&str>) -> String {
    let mut card: Value = serde_json::from_str(CARD).unwrap();
    card["id"] = Value::String(Uuid::from_u128(0xc0de_0000 | n).to_string());
    card["oracle_id"] = Value::String(Uuid::from_u128(0x0a1c_0000 | oracle).to_string());
    card["name"] = Value::String(format!("Fixture Card {oracle}"));
    card["collector_number"] = Value::String(n.to_string());
    // Unique per printing in the catalog; the template's would collide.
    card["mtgo_id"] = Value::Null;
    card["mtgo_foil_id"] = Value::Null;
    card["prices"]["usd"] = usd.map_or(Value::Null, |p| Value::String(p.to_string()));
    card.to_string()
}

/// A card file of printings `0..count`, one card each, every one priced.
fn card_file(count: u128) -> String {
    (0..count)
        .map(|n| card_line(n, n, Some("1.00")))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The card catalog's printings, priced printings and last card load.
async fn catalog(pool: &sqlx::PgPool) -> (i64, i64, Option<DateTime<Utc>>) {
    let (printings, priced): (i64, i64) = sqlx::query_as(
        "SELECT count(*), count(*) FILTER (WHERE prices->>'usd' IS NOT NULL) FROM scryfall_data",
    )
    .fetch_one(pool)
    .await
    .unwrap();
    let mut conn = pool.acquire().await.unwrap();
    let load = scryfall_loads::fetch_load(&mut conn, "default_cards")
        .await
        .unwrap();
    (printings, priced, load)
}

/// Syncs a good file of `count` printings made on day 19, the catalog later tests start from.
async fn seed_catalog(pool: &sqlx::PgPool, count: u128) {
    let fixture = Fixture::new(BulkEndpoint::DefaultCards, &card_file(count), day(19));
    cards(pool).scryfall_sync(&fixture).await.unwrap();
    assert_eq!(
        catalog(pool).await,
        (
            i64::try_from(count).unwrap(),
            i64::try_from(count).unwrap(),
            Some(day(19))
        )
    );
}

/// Syncs `fixture` and asserts it is refused as `expected` with the catalog left as it was.
async fn assert_refused(
    pool: &sqlx::PgPool,
    fixture: &Fixture,
    expected: fn(&RefusedPull) -> bool,
) {
    let before = catalog(pool).await;
    let error = cards(pool).scryfall_sync(fixture).await.unwrap_err();
    assert!(refusal(&error).is_some_and(expected), "{error:#}");
    assert_eq!(catalog(pool).await, before, "the stored catalog stays");
}

#[sqlx::test]
async fn a_good_card_file_loads_and_records_its_date(pool: sqlx::PgPool) {
    seed_catalog(&pool, 10).await;
    let again = Fixture::new(BulkEndpoint::DefaultCards, &card_file(10), day(20));
    let synced = cards(&pool).scryfall_sync(&again).await.unwrap();
    assert_eq!(synced.printing_ids.len(), 10);
    assert_eq!(catalog(&pool).await, (10, 10, Some(day(20))));
}

#[sqlx::test]
async fn an_empty_card_file_is_refused(pool: sqlx::PgPool) {
    seed_catalog(&pool, 10).await;
    let empty = Fixture::new(BulkEndpoint::DefaultCards, "", day(20));
    assert_refused(&pool, &empty, |r| matches!(r, RefusedPull::Empty { .. })).await;
}

#[sqlx::test]
async fn a_card_file_under_ninety_percent_of_the_catalog_is_refused(pool: sqlx::PgPool) {
    seed_catalog(&pool, 10).await;
    let short = Fixture::new(BulkEndpoint::DefaultCards, &card_file(8), day(20));
    assert_refused(&pool, &short, |r| matches!(r, RefusedPull::Shrunk { .. })).await;
    let at_the_line = Fixture::new(BulkEndpoint::DefaultCards, &card_file(9), day(20));
    cards(&pool).scryfall_sync(&at_the_line).await.unwrap();
}

#[sqlx::test]
async fn a_card_file_listing_a_printing_twice_or_the_nil_id_is_refused(pool: sqlx::PgPool) {
    seed_catalog(&pool, 10).await;
    let twice = format!("{}\n{}", card_file(10), card_line(3, 3, Some("1.00")));
    let doubled = Fixture::new(BulkEndpoint::DefaultCards, &twice, day(20));
    assert_refused(&pool, &doubled, |r| {
        matches!(r, RefusedPull::DuplicateIds { count: 1 })
    })
    .await;

    let mut nil: Value = serde_json::from_str(&card_line(10, 10, None)).unwrap();
    nil["id"] = Value::String(Uuid::nil().to_string());
    let with_nil = format!("{}\n{nil}", card_file(10));
    let niled = Fixture::new(BulkEndpoint::DefaultCards, &with_nil, day(20));
    assert_refused(&pool, &niled, |r| {
        matches!(r, RefusedPull::NilIds { count: 1 })
    })
    .await;
}

#[sqlx::test]
async fn a_card_file_that_drops_most_prices_is_refused(pool: sqlx::PgPool) {
    seed_catalog(&pool, 10).await;
    let wiped: String = (0..10)
        .map(|n| card_line(n, n, (n < 4).then_some("2.00")))
        .collect::<Vec<_>>()
        .join("\n");
    let fixture = Fixture::new(BulkEndpoint::DefaultCards, &wiped, day(20));
    assert_refused(&pool, &fixture, |r| {
        matches!(r, RefusedPull::PricesWiped { .. })
    })
    .await;

    let half: String = (0..10)
        .map(|n| card_line(n, n, (n < 5).then_some("2.00")))
        .collect::<Vec<_>>()
        .join("\n");
    let at_the_line = Fixture::new(BulkEndpoint::DefaultCards, &half, day(20));
    cards(&pool).scryfall_sync(&at_the_line).await.unwrap();
    assert_eq!(catalog(&pool).await.1, 5);
}

#[sqlx::test]
async fn an_older_card_file_than_the_last_loaded_is_refused(pool: sqlx::PgPool) {
    seed_catalog(&pool, 10).await;
    let older = Fixture::new(BulkEndpoint::DefaultCards, &card_file(10), day(18));
    assert_refused(&pool, &older, |r| matches!(r, RefusedPull::Stale { .. })).await;
}

#[sqlx::test]
async fn a_truncated_card_download_is_refused(pool: sqlx::PgPool) {
    seed_catalog(&pool, 10).await;
    let good = Fixture::new(BulkEndpoint::DefaultCards, &card_file(10), day(20));
    let cut = good.bytes.get(..good.bytes.len() / 2).unwrap().to_vec();
    let truncated = good.serving(cut);
    assert_refused(&pool, &truncated, |r| {
        matches!(r, RefusedPull::Truncated { .. })
    })
    .await;
}

#[sqlx::test]
async fn a_removal_over_its_cap_is_refused_and_removes_nothing(pool: sqlx::PgPool) {
    seed_catalog(&pool, 600).await;
    let short: HashSet<Uuid> = (0..50).map(|n| Uuid::from_u128(0xc0de_0000 | n)).collect();
    let error = cards(&pool)
        .remove_deleted_printings(&short, RemovalMode::Apply)
        .await
        .unwrap_err();
    assert!(
        matches!(
            refusal(&error),
            Some(RefusedPull::MassRemoval {
                removing: 550,
                cap: 500
            })
        ),
        "{error:#}"
    );
    // A preview lists the whole plan and says it would be refused.
    let preview = cards(&pool)
        .remove_deleted_printings(&short, RemovalMode::Preview)
        .await
        .unwrap();
    assert_eq!(preview.plan.removals.len(), 550);
    assert!(matches!(
        preview.refused,
        Some(RefusedPull::MassRemoval { .. })
    ));
    assert_eq!(catalog(&pool).await.0, 600);

    let most: HashSet<Uuid> = (0..100).map(|n| Uuid::from_u128(0xc0de_0000 | n)).collect();
    let report = cards(&pool)
        .remove_deleted_printings(&most, RemovalMode::Apply)
        .await
        .unwrap();
    assert_eq!(
        report.plan.removals.len(),
        500,
        "exactly the cap goes through"
    );
    assert_eq!(report.refused, None);
}

// == the oracle tag sync and the rebuilds ==

/// An oracle tags file: each `(slug, cards)` a tag on the fixture cards numbered in `cards`.
fn tags_file(tags: &[(&str, std::ops::Range<u128>)]) -> String {
    tags.iter()
        .enumerate()
        .map(|(i, (slug, cards))| {
            let taggings: Vec<Value> = cards
                .clone()
                .map(|n| serde_json::json!({ "oracle_id": Uuid::from_u128(0x0a1c_0000 | n) }))
                .collect();
            serde_json::json!({
                "object": "tag",
                "id": Uuid::from_u128(0x7a90_0000 | u128::try_from(i).unwrap()),
                "slug": slug,
                "label": slug,
                "parent_ids": [],
                "taggings": taggings,
            })
            .to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The first two role roots, so the derived roles and the grouping have something to show.
fn roots() -> (&'static str, &'static str) {
    (CATEGORY_ROOTS[0].1[0], CATEGORY_ROOTS[1].1[0])
}

/// A good tag file for the ten fixture cards: one root on all ten, the other on five, and a
/// third tag on all ten.
fn good_tags() -> String {
    let (first, second) = roots();
    tags_file(&[(first, 0..10), (second, 0..5), ("fixture-extra", 0..10)])
}

/// Stored tags, card tags, and the profiles carrying projected tags, tag groups and roles.
async fn tag_state(pool: &sqlx::PgPool) -> (i64, i64, i64, i64, i64) {
    sqlx::query_as(
        "SELECT (SELECT count(*) FROM oracle_tags),
                (SELECT count(*) FROM card_oracle_tags WHERE source = 'scryfall'),
                (SELECT count(*) FROM card_profiles WHERE oracle_tags <> '[]'::jsonb),
                (SELECT count(*) FROM card_profiles WHERE oracle_tags_by_role <> '{}'::jsonb),
                (SELECT count(*) FROM card_profiles WHERE card_roles <> '[]'::jsonb)",
    )
    .fetch_one(pool)
    .await
    .unwrap()
}

/// The steps after the tag sync, in zervice's order: the projection, the grouping and the roles.
async fn rebuild(pool: &sqlx::PgPool) -> [anyhow::Result<()>; 3] {
    let service = cards(pool);
    [
        service.refresh_card_oracle_tags().await.map(drop),
        service.refresh_oracle_tag_groups().await.map(drop),
        service.derive_card_categories(100).await.map(drop),
    ]
}

/// Ten cards with good tags synced on day 19 and every rebuild run.
async fn seed_tagged(pool: &sqlx::PgPool) -> (i64, i64, i64, i64, i64) {
    seed_catalog(pool, 10).await;
    let fixture = Fixture::new(BulkEndpoint::OracleTags, &good_tags(), day(19));
    cards(pool).sync_oracle_tags(&fixture).await.unwrap();
    for step in rebuild(pool).await {
        step.unwrap();
    }
    let state = tag_state(pool).await;
    assert_eq!(
        state,
        (3, 25, 10, 10, 10),
        "every card tagged, grouped and given a role"
    );
    state
}

#[sqlx::test]
async fn a_good_tag_file_loads_and_records_its_date(pool: sqlx::PgPool) {
    seed_tagged(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    assert_eq!(
        scryfall_loads::fetch_load(&mut conn, "oracle_tags")
            .await
            .unwrap(),
        Some(day(19))
    );
}

#[sqlx::test]
async fn the_empty_tag_file_fails_its_night_and_every_role_stays(pool: sqlx::PgPool) {
    let tagged = seed_tagged(&pool).await;
    let empty = Fixture::new(BulkEndpoint::OracleTags, "", day(20));
    let error = cards(&pool).sync_oracle_tags(&empty).await.unwrap_err();
    assert!(
        matches!(refusal(&error), Some(RefusedPull::Empty { .. })),
        "{error:#}"
    );
    // zervice skips the projection and the grouping after a failed sync; the roles still run.
    cards(&pool).derive_card_categories(100).await.unwrap();
    assert_eq!(tag_state(&pool).await, tagged);
}

#[sqlx::test]
async fn a_tag_file_under_eighty_percent_or_older_is_refused(pool: sqlx::PgPool) {
    let tagged = seed_tagged(&pool).await;
    let (first, second) = roots();
    let short = Fixture::new(
        BulkEndpoint::OracleTags,
        &tags_file(&[(first, 0..10), (second, 0..5)]),
        day(20),
    );
    let error = cards(&pool).sync_oracle_tags(&short).await.unwrap_err();
    assert!(
        matches!(refusal(&error), Some(RefusedPull::Shrunk { .. })),
        "{error:#}"
    );

    let older = Fixture::new(BulkEndpoint::OracleTags, &good_tags(), day(18));
    let error = cards(&pool).sync_oracle_tags(&older).await.unwrap_err();
    assert!(
        matches!(refusal(&error), Some(RefusedPull::Stale { .. })),
        "{error:#}"
    );
    assert_eq!(tag_state(&pool).await, tagged);
}

#[sqlx::test]
async fn a_restore_from_an_older_file_loads_and_never_lowers_the_mark(pool: sqlx::PgPool) {
    seed_tagged(&pool).await;
    let older = Fixture::new(BulkEndpoint::OracleTags, &good_tags(), day(18));
    cards(&pool)
        .sync_oracle_tags_from(
            &older,
            "https://data.scryfall.io/oracle-tags/older.jsonl.gz",
        )
        .await
        .unwrap();
    let mut conn = pool.acquire().await.unwrap();
    assert_eq!(
        scryfall_loads::fetch_load(&mut conn, "oracle_tags")
            .await
            .unwrap(),
        Some(day(19))
    );
}

#[sqlx::test]
async fn every_rebuild_refuses_empty_tag_tables_and_keeps_its_column(pool: sqlx::PgPool) {
    let tagged = seed_tagged(&pool).await;
    sqlx::query("DELETE FROM card_oracle_tags")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM oracle_tags")
        .execute(&pool)
        .await
        .unwrap();
    for step in rebuild(&pool).await {
        let error = step.unwrap_err();
        assert!(
            matches!(refusal(&error), Some(RefusedPull::NoInput { .. })),
            "{error:#}"
        );
    }
    let (_, _, projected, grouped, roles) = tag_state(&pool).await;
    assert_eq!((projected, grouped, roles), (tagged.2, tagged.3, tagged.4));
}

#[sqlx::test]
async fn a_rebuild_that_would_strip_most_profiles_rolls_back(pool: sqlx::PgPool) {
    let tagged = seed_tagged(&pool).await;
    // Seven of the ten cards lose every tag row: each rebuild would leave three of ten.
    sqlx::query("DELETE FROM card_oracle_tags WHERE oracle_id <> ALL($1)")
        .bind(
            (0..3)
                .map(|n| Uuid::from_u128(0x0a1c_0000 | n))
                .collect::<Vec<_>>(),
        )
        .execute(&pool)
        .await
        .unwrap();
    for step in rebuild(&pool).await {
        let error = step.unwrap_err();
        assert!(
            matches!(
                refusal(&error),
                Some(RefusedPull::RebuildShrunk {
                    after: 3,
                    before: 10,
                    ..
                })
            ),
            "{error:#}"
        );
    }
    let (_, _, projected, grouped, roles) = tag_state(&pool).await;
    assert_eq!((projected, grouped, roles), (tagged.2, tagged.3, tagged.4));
}
