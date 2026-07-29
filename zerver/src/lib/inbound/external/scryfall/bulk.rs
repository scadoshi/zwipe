use crate::inbound::external::scryfall::{
    oracle_tag::OracleTag,
    planeswalker::{Planeswalker, SCRYFALL_API_BASE},
};
use anyhow::Context;
use flate2::read::GzDecoder;
use reqwest::Client;
use serde::{Deserialize, de::DeserializeOwned};
use std::io::{BufRead, BufReader};
use zwipe_core::domain::card::scryfall_data::ScryfallData;

/// Scryfall bulk data metadata response (contains the download URI).
///
/// Scryfall retired the plain-JSON `download_uri` in late July 2026; bulk
/// files are now served exclusively as gzipped JSON Lines via
/// `jsonl_download_uri` (one object per line). Line-by-line parsing also
/// avoids the old whole-array-in-memory spike.
#[derive(Deserialize, Debug)]
pub(super) struct BulkDataObject {
    pub(super) jsonl_download_uri: String,
}

/// Scryfall's bulk data download categories.
#[derive(Debug, Clone, Copy)]
#[allow(missing_docs)]
pub enum BulkEndpoint {
    OracleCards,
    UniqueArtwork,
    DefaultCards,
    AllCards,
    Rulings,
    OracleTags,
}

impl BulkEndpoint {
    pub(super) fn resolve(&self) -> String {
        match self {
            Self::OracleCards => "/bulk-data/oracle-cards".to_string(),
            Self::UniqueArtwork => "/bulk-data/unique-artworks".to_string(),
            Self::DefaultCards => "/bulk-data/default-cards".to_string(),
            Self::AllCards => "/bulk-data/all-cards".to_string(),
            Self::Rulings => "/bulk-data/rulings".to_string(),
            Self::OracleTags => "/bulk-data/oracle-tags".to_string(),
        }
    }

    /// Returns the snake_case name for file naming.
    pub fn to_snake_case(&self) -> String {
        match self {
            Self::OracleCards => "oracle_cards".to_string(),
            Self::UniqueArtwork => "unique_artwork".to_string(),
            Self::DefaultCards => "default_cards".to_string(),
            Self::AllCards => "all_cards".to_string(),
            Self::Rulings => "rulings".to_string(),
            Self::OracleTags => "oracle_tags".to_string(),
        }
    }

    /// Two-step bulk fetch: metadata endpoint → `jsonl_download_uri` →
    /// gunzip → one parsed `T` per line. Blank lines are skipped; a malformed
    /// line fails the whole fetch (better a loud sync failure than silent
    /// partial data).
    async fn amass_jsonl<T: DeserializeOwned>(&self, what: &str) -> anyhow::Result<Vec<T>> {
        let url = format!("{}{}", SCRYFALL_API_BASE, self.resolve());
        let urza = Planeswalker::untap(Client::new(), &url);

        let bulk_data_object: BulkDataObject = urza
            .cast()
            .await
            .with_context(|| format!("failed to get {what} bulk response with planeswalker"))?
            .json()
            .await
            .with_context(|| format!("failed to parse BulkDataObject for {what}"))?;

        let karn = Planeswalker::untap(Client::new(), &bulk_data_object.jsonl_download_uri);
        let bytes = karn
            .cast()
            .await
            .with_context(|| format!("failed to get {what} download response with planeswalker"))?
            .bytes()
            .await
            .with_context(|| format!("failed to read {what} download body"))?;

        parse_jsonl_gz(&bytes).with_context(|| format!("failed to parse {what} jsonl"))
    }

    /// Fetches bulk card data (`Vec<ScryfallData>`).
    pub async fn amass(&self) -> anyhow::Result<Vec<ScryfallData>> {
        self.amass_jsonl(&self.to_snake_case()).await
    }

    /// Fetches the Oracle Tags bulk file (`Vec<OracleTag>`).
    pub async fn amass_oracle_tags(&self) -> anyhow::Result<Vec<OracleTag>> {
        self.amass_jsonl("oracle-tags").await
    }
}

/// Decompresses a gzipped JSON Lines payload and parses each non-blank line.
fn parse_jsonl_gz<T: DeserializeOwned>(bytes: &[u8]) -> anyhow::Result<Vec<T>> {
    let reader = BufReader::new(GzDecoder::new(bytes));
    let mut items = Vec::new();
    for (i, line) in reader.lines().enumerate() {
        let line = line.with_context(|| format!("failed to read jsonl line {}", i + 1))?;
        if line.trim().is_empty() {
            continue;
        }
        items.push(
            serde_json::from_str(&line)
                .with_context(|| format!("failed to parse jsonl line {}", i + 1))?,
        );
    }
    Ok(items)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::parse_jsonl_gz;
    use flate2::{Compression, write::GzEncoder};
    use std::io::Write;

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
    fn parses_one_object_per_line_skipping_blanks() {
        let bytes = gz("{\"name\":\"a\"}\n\n{\"name\":\"b\"}\n");
        let rows: Vec<Row> = parse_jsonl_gz(&bytes).unwrap();
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
        let err = parse_jsonl_gz::<Row>(&bytes).unwrap_err();
        assert!(format!("{err:#}").contains("line 2"), "{err:#}");
    }
}
