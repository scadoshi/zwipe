//! Bulk data download endpoints.

use std::io::{BufRead, BufReader};

use anyhow::Context;
use chrono::{DateTime, Utc};
use flate2::read::GzDecoder;
use reqwest::Client;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use zwipe_core::domain::card::scryfall_data::ScryfallData;

use crate::domain::card::models::scryfall_pull;
use crate::inbound::external::scryfall::request::{SCRYFALL_API_BASE, ScryfallRequest};

/// Scryfall bulk data metadata response (contains the download URI).
///
/// Scryfall serves bulk files only as gzipped JSON Lines, via
/// `jsonl_download_uri` (one object per line). Line-by-line parsing also
/// avoids holding the whole array in memory.
#[derive(Deserialize, Debug)]
pub(super) struct BulkDataObject {
    pub(super) jsonl_download_uri: String,
    /// When Scryfall made the file.
    #[serde(default)]
    pub(super) updated_at: Option<DateTime<Utc>>,
    /// The file's size in bytes.
    #[serde(default)]
    pub(super) compressed_size: Option<u64>,
}

/// Scryfall's bulk data download categories.
#[derive(Debug, Clone, Copy)]
#[expect(
    missing_docs,
    reason = "each variant names the Scryfall bulk file it fetches"
)]
pub enum BulkEndpoint {
    OracleCards,
    UniqueArtwork,
    DefaultCards,
    AllCards,
    Rulings,
    OracleTags,
}

impl BulkEndpoint {
    pub(super) fn resolve(self) -> String {
        match self {
            Self::OracleCards => "/bulk-data/oracle-cards".to_string(),
            Self::UniqueArtwork => "/bulk-data/unique-artworks".to_string(),
            Self::DefaultCards => "/bulk-data/default-cards".to_string(),
            Self::AllCards => "/bulk-data/all-cards".to_string(),
            Self::Rulings => "/bulk-data/rulings".to_string(),
            Self::OracleTags => "/bulk-data/oracle-tags".to_string(),
        }
    }

    /// The bulk file's name, as Scryfall types it and `scryfall_loads` keys it.
    #[must_use]
    pub fn kind(self) -> &'static str {
        match self {
            Self::OracleCards => "oracle_cards",
            Self::UniqueArtwork => "unique_artwork",
            Self::DefaultCards => "default_cards",
            Self::AllCards => "all_cards",
            Self::Rulings => "rulings",
            Self::OracleTags => "oracle_tags",
        }
    }
}

/// One bulk file as its source lists it: where to download it, when Scryfall made it and how
/// big it is. A file named by URL alone carries neither, so neither check applies to it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BulkFile {
    /// The bulk file's name, [`BulkEndpoint::kind`].
    pub kind: &'static str,
    /// Where it downloads from.
    pub uri: String,
    /// When Scryfall made it.
    pub updated_at: Option<DateTime<Utc>>,
    /// Its size as Scryfall lists it, in bytes.
    pub compressed_size: Option<u64>,
}

/// Where bulk files come from: Scryfall in a run, fixed bytes in a test.
pub trait BulkSource: Sync {
    /// Lists `endpoint`'s current file.
    ///
    /// # Errors
    ///
    /// Fails when the listing can't be read.
    fn describe(
        &self,
        endpoint: BulkEndpoint,
    ) -> impl Future<Output = anyhow::Result<BulkFile>> + Send;

    /// Downloads the file at `uri`, whole.
    ///
    /// # Errors
    ///
    /// Fails when the download can't be read or is refused.
    fn download(&self, uri: &str) -> impl Future<Output = anyhow::Result<Vec<u8>>> + Send;
}

/// Scryfall's API and its bulk file host.
#[derive(Debug, Clone, Copy, Default)]
pub struct Scryfall;

impl BulkSource for Scryfall {
    async fn describe(&self, endpoint: BulkEndpoint) -> anyhow::Result<BulkFile> {
        let kind = endpoint.kind();
        let url = format!("{}{}", SCRYFALL_API_BASE, endpoint.resolve());
        let object: BulkDataObject = ScryfallRequest::get(&Client::new(), &url)
            .send()
            .await
            .with_context(|| format!("failed to get {kind} bulk response with planeswalker"))?
            .error_for_status()
            .with_context(|| format!("Scryfall refused the {kind} bulk listing"))?
            .json()
            .await
            .with_context(|| format!("failed to parse BulkDataObject for {kind}"))?;
        Ok(BulkFile {
            kind,
            uri: object.jsonl_download_uri,
            updated_at: object.updated_at,
            compressed_size: object.compressed_size,
        })
    }

    async fn download(&self, uri: &str) -> anyhow::Result<Vec<u8>> {
        let bytes = ScryfallRequest::get(&Client::new(), uri)
            .send()
            .await
            .with_context(|| format!("failed to get {uri} with planeswalker"))?
            .error_for_status()
            .with_context(|| format!("Scryfall refused the download of {uri}"))?
            .bytes()
            .await
            .with_context(|| format!("failed to read the download of {uri}"))?;
        Ok(bytes.to_vec())
    }
}

/// A bulk file's items with what its source listed for it.
#[derive(Debug, Clone)]
pub struct Pull<T> {
    /// The file.
    pub file: BulkFile,
    /// One item per line.
    pub items: Vec<T>,
}

/// Downloads `file` from `source`, checks its size against the listing and parses one `T` per
/// line. Blank lines are skipped; a line that isn't a `T` fails the whole pull.
///
/// # Errors
///
/// Fails when the download fails, its size differs from the listing
/// ([`scryfall_pull::RefusedPull::Truncated`]), the gzip is broken or any line doesn't parse.
pub async fn pull<T: DeserializeOwned, S: BulkSource>(
    source: &S,
    file: BulkFile,
) -> anyhow::Result<Pull<T>> {
    let bytes = source.download(&file.uri).await?;
    scryfall_pull::check_size(
        file.kind,
        u64::try_from(bytes.len()).unwrap_or(u64::MAX),
        file.compressed_size,
    )?;
    let items =
        parse_jsonl_gz(&bytes).with_context(|| format!("failed to parse {} jsonl", file.kind))?;
    Ok(Pull { file, items })
}

/// Pulls the card file, lifting each reversible card's `oracle_id` from its faces, where
/// Scryfall puts it, before the checks and the upsert see the card.
///
/// # Errors
///
/// As [`pull`].
pub async fn pull_cards<S: BulkSource>(
    source: &S,
    file: BulkFile,
) -> anyhow::Result<Pull<ScryfallData>> {
    let mut pulled: Pull<ScryfallData> = pull(source, file).await?;
    for card in &mut pulled.items {
        card.backfill_oracle_id_from_faces();
    }
    Ok(pulled)
}

/// The host Scryfall serves its dated bulk files from, the one host a file named by URL may be.
pub const BULK_FILE_HOST: &str = "data.scryfall.io";

/// The dated Oracle Tags file at `url`, such as an earlier night's when the latest one is bad.
///
/// # Errors
///
/// Fails when the URL isn't https on [`BULK_FILE_HOST`].
pub fn oracle_tags_file(url: &str) -> anyhow::Result<BulkFile> {
    check_bulk_file_url(url)?;
    Ok(BulkFile {
        kind: BulkEndpoint::OracleTags.kind(),
        uri: url.to_string(),
        updated_at: None,
        compressed_size: None,
    })
}

/// Accepts only an https URL on [`BULK_FILE_HOST`].
///
/// # Errors
///
/// Fails when `url` doesn't parse, isn't https, or names another host.
pub fn check_bulk_file_url(url: &str) -> anyhow::Result<()> {
    let parsed = reqwest::Url::parse(url).with_context(|| format!("not a URL: {url}"))?;
    anyhow::ensure!(
        parsed.scheme() == "https" && parsed.host_str() == Some(BULK_FILE_HOST),
        "a bulk file comes from https://{BULK_FILE_HOST}/, not {url}"
    );
    Ok(())
}

/// Decompresses a gzipped JSON Lines payload and parses each non-blank line.
fn parse_jsonl_gz<T: DeserializeOwned>(bytes: &[u8]) -> anyhow::Result<Vec<T>> {
    let reader = BufReader::new(GzDecoder::new(bytes));
    let mut items = Vec::new();
    for (i, line) in reader.lines().enumerate() {
        let line =
            line.with_context(|| format!("failed to read jsonl line {}", i.saturating_add(1)))?;
        if line.trim().is_empty() {
            continue;
        }
        items.push(
            serde_json::from_str(&line)
                .with_context(|| format!("failed to parse jsonl line {}", i.saturating_add(1)))?,
        );
    }
    Ok(items)
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use flate2::Compression;
    use flate2::write::GzEncoder;

    fn gz(lines: &str) -> Vec<u8> {
        let mut enc = GzEncoder::new(Vec::new(), Compression::default());
        enc.write_all(lines.as_bytes()).unwrap();
        enc.finish().unwrap()
    }

    #[derive(serde::Deserialize, Debug, PartialEq)]
    struct Row {
        name: String,
    }

    #[test]
    fn a_file_url_must_be_https_on_scryfalls_bulk_host() {
        let good = "https://data.scryfall.io/oracle-tags/oracle-tags-20261118220038.jsonl.gz";
        assert!(super::check_bulk_file_url(good).is_ok());
        for bad in [
            "http://data.scryfall.io/oracle-tags/oracle-tags-20261118220038.jsonl.gz",
            "https://data.scryfall.io.example.com/oracle-tags.jsonl.gz",
            "https://example.com/oracle-tags-20261118220038.jsonl.gz",
            "file:///tmp/oracle-tags.jsonl.gz",
            "data.scryfall.io/oracle-tags.jsonl.gz",
        ] {
            assert!(super::check_bulk_file_url(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn parses_one_object_per_line_skipping_blanks() {
        let bytes = gz("{\"name\":\"a\"}\n\n{\"name\":\"b\"}\n");
        let rows: Vec<Row> = super::parse_jsonl_gz(&bytes).unwrap();
        assert_eq!(
            rows,
            vec![
                Row {
                    name: "a".to_string()
                },
                Row {
                    name: "b".to_string()
                }
            ]
        );
    }

    #[test]
    fn malformed_line_fails_loudly_with_line_number() {
        let bytes = gz("{\"name\":\"a\"}\nnot json\n");
        let err = super::parse_jsonl_gz::<Row>(&bytes).unwrap_err();
        assert!(format!("{err:#}").contains("line 2"), "{err:#}");
    }
}
