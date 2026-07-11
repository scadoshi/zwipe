use crate::inbound::external::scryfall::{
    oracle_tag::OracleTag,
    planeswalker::{Planeswalker, SCRYFALL_API_BASE},
};
use anyhow::Context;
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use zwipe_core::domain::card::scryfall_data::ScryfallData;

/// Scryfall bulk data metadata response (contains the download URI).
#[derive(Deserialize, Debug)]
pub(super) struct BulkDataObject {
    pub(super) download_uri: String,
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
}

impl BulkEndpoint {
    /// Fetches bulk card data in two steps: metadata endpoint → download URI → card data.
    pub async fn amass(&self) -> anyhow::Result<Vec<ScryfallData>> {
        // first get the bulk data object with our main url
        let url = format!("{}{}", SCRYFALL_API_BASE, self.resolve());
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

        let cards: Vec<ScryfallData> =
            serde_json::from_value(cards_json).context("failed to parse `Vec<ScryfallData>`")?;

        Ok(cards)
    }

    /// Fetches the Oracle Tags bulk file in the same two steps as [`amass`], but
    /// parses the tag payload (`Vec<OracleTag>`) instead of card data.
    ///
    /// [`amass`]: BulkEndpoint::amass
    pub async fn amass_oracle_tags(&self) -> anyhow::Result<Vec<OracleTag>> {
        // first get the bulk data object with our main url
        let url = format!("{}{}", SCRYFALL_API_BASE, self.resolve());
        let urza = Planeswalker::untap(Client::new(), &url);

        let bulk_response = urza
            .cast()
            .await
            .context("failed to get oracle-tags bulk response with planeswalker")?;

        let bulk_json: Value = bulk_response
            .json()
            .await
            .context("failed to parse json from oracle-tags main uri result")?;

        let bulk_data_object = serde_json::from_value::<BulkDataObject>(bulk_json)
            .context("failed to parse BulkDataObject for oracle-tags")?;

        // then use the download_uri to fetch the actual tag data
        let karn = Planeswalker::untap(Client::new(), &bulk_data_object.download_uri);

        let tags_response = karn
            .cast()
            .await
            .context("failed to get oracle-tags download response with planeswalker")?;

        let tags_json: Value = tags_response
            .json()
            .await
            .context("failed to parse json from oracle-tags download uri result")?;

        let tags: Vec<OracleTag> =
            serde_json::from_value(tags_json).context("failed to parse `Vec<OracleTag>`")?;

        Ok(tags)
    }
}
