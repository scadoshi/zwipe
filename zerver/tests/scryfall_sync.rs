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
    reason = "a test: helpers outside #[test] fns unwrap, and a panic is the failure report"
)]

mod common;

use std::future::{self, Future};
use std::io::Write;

use chrono::{DateTime, TimeZone, Utc};
use flate2::Compression;
use flate2::write::GzEncoder;
use zwipe::domain::card::models::scryfall_pull::{RefusedPull, ScryfallLoad};
use zwipe::inbound::external::scryfall::bulk::{self, BulkEndpoint, BulkFile, BulkSource};
use zwipe::inbound::external::scryfall::oracle_tag::OracleTag;
use zwipe::outbound::sqlx::card::helpers::scryfall_loads;

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
