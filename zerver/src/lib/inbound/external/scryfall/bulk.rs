use std::fs::read;

use crate::{
    domain::card::models::scryfall_data::ScryfallData,
    inbound::external::scryfall::planeswalker::{Planeswalker, SCRYFALL_API_BASE},
};
use anyhow::Context;
use reqwest::Client;
use serde::Deserialize;
use serde_json::{from_slice, from_value, Value};

/// scryfall returns this
/// when you get bulk data
#[derive(Deserialize, Debug)]
pub(super) struct BulkDataObject {
    // content_encoding: String,
    // content_type: String,
    // description: String,
    pub(super) download_uri: String,
    // id: String,
    // name: String,
    // object: String,
    // size: i64,
    // #[serde(rename = "type")]
    // bulk_type: String,
    // updated_at: String,
    // uri: String,
}

// ==========================
//  bulk endpoint ergonomics
// ==========================
// not sure if every endpoints returns consistent types
// only tested `OracleCards` for now :O

#[derive(Debug, Clone, Copy)]
pub enum BulkEndpoint {
    OracleCards,
    UniqueArtwork,
    DefaultCards,
    AllCards,
    Rulings,
}

impl BulkEndpoint {
    pub(super) fn resolve(&self) -> String {
        match self {
            Self::OracleCards => "/bulk-data/oracle-cards".to_string(),
            Self::UniqueArtwork => "/bulk-data/unique-artworks".to_string(),
            Self::DefaultCards => "/bulk-data/default-cards".to_string(),
            Self::AllCards => "/bulk-data/all-cards".to_string(),
            Self::Rulings => "/bulk-data/rulings".to_string(),
        }
    }

    pub fn to_snake_case(&self) -> String {
        match self {
            Self::OracleCards => "oracle_cards".to_string(),
            Self::UniqueArtwork => "unique_artwork".to_string(),
            Self::DefaultCards => "default_cards".to_string(),
            Self::AllCards => "all_cards".to_string(),
            Self::Rulings => "rulings".to_string(),
        }
    }
}

pub const CACHE_BULK_PATH: &str = "cache/bulk/";
impl BulkEndpoint {
    /// gets bulk cards with a BulkEndpoint parameter end returns `Vec<ScryfallData>`
    pub async fn amass(&self, should_use_cache: bool) -> anyhow::Result<Vec<ScryfallData>> {
        let mut should_update_cache: bool = false;
        let full_cache_path = format!("{},{}.json", CACHE_BULK_PATH, self.to_snake_case());
        if should_use_cache {
            let error_encountered: bool;

            let try_parse_cache = || -> anyhow::Result<Vec<ScryfallData>> {
                let bytes = read(&full_cache_path)?;
                let cards_json = from_slice::<Value>(&bytes)?;
                Ok(from_value::<Vec<ScryfallData>>(cards_json)?)
            };

            match try_parse_cache() {
                Ok(cards) => return Ok(cards),
                Err(e) => {
                    error_encountered = true;
                    tracing::warn!("failed to parse cards: {}", e);
                }
            }

            if error_encountered {
                should_update_cache = true;
            }
        } else {
            tracing::info!("cache mode is turned off; falling back to api")
        }

        // first get the bulk data object with our main url
        let url = format!("{}{}", SCRYFALL_API_BASE, &self.resolve());
        let urza = Planeswalker::untap(Client::new(), &url);

        let bulk_response = urza
            .cast()
            .await
            .context("failed to get bulk response with planeswalker")?;

        let bulk_json: Value = bulk_response
            .json()
            .await
            .context("failed to parse json from main uri result")?;

        let bulk_data_object = serde_json::from_value::<BulkDataObject>(bulk_json)
            .context("failed to parse BulkDataObject")?;

        // then use the download_uri to fetch the actual card data
        let karn = Planeswalker::untap(Client::new(), &bulk_data_object.download_uri);

        let cards_response = karn
            .cast()
            .await
            .context("failed to get download response with planeswalker")?;

        let cards_json: Value = cards_response
            .json()
            .await
            .context("failed to parse json from download uri result")?;

        if should_use_cache && should_update_cache {
            // ensure all directories of cache path are in place
            let Some(cache_file_parent_directory) = std::path::Path::new(&full_cache_path).parent()
            else {
                return Err(anyhow::anyhow!(
                    "unable to derive parent directory from {}",
                    full_cache_path
                ));
            };
            std::fs::create_dir_all(cache_file_parent_directory)
                .context("failed to create bulk directories")?;
            // then attempt writing to it
            std::fs::write(
                &full_cache_path,
                serde_json::to_string(&cards_json)
                    .context("failed to write cards_json to string for bulk caching")?,
            )
            .context("failed to write to cache file")?;
            tracing::info!("successfully wrote to cache file");
        }

        let cards: Vec<ScryfallData> =
            serde_json::from_value(cards_json).context("failed to parse `Vec<ScryfallData>`")?;

        Ok(cards)
    }
}
