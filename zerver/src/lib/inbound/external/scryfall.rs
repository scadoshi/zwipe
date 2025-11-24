use anyhow::Context;
use reqwest::{
    header::{ACCEPT, USER_AGENT},
    Client, RequestBuilder, Response,
};
use serde::Deserialize;
use serde_json::{from_value, Value};

use crate::domain::card::models::scryfall_data::ScryfallData;

// ==============================
//  equip onto scryfall requests
// ==============================
const USER_AGENT_VALUE: &str = "zwipe/0.0";
const ACCEPT_VALUE: &str = "*/*";
const SCRYFALL_API_BASE: &str = "https://api.scryfall.com";
const CARDS_SEARCH_ENDPOINT: &str = "/cards/search";

/// scryfall returns this
/// when you search for a card
#[derive(Deserialize, Debug)]
struct ScryfallDataSearchResponse {
    data: Vec<ScryfallData>,
    // has_more: bool,
    // object: String,
    // total_cards: i32,
}

/// scryfall returns this
/// when you get bulk data
#[derive(Deserialize, Debug)]
struct BulkDataObject {
    // content_encoding: String,
    // content_type: String,
    // description: String,
    download_uri: String,
    // id: String,
    // name: String,
    // object: String,
    // size: i64,
    // #[serde(rename = "type")]
    // bulk_type: String,
    // updated_at: String,
    // uri: String,
}

// =========
//  helpers
// =========

// for building scryfall based requests
// yes these use magic the gathering terms

#[derive(Debug)]
/// ensures we have a `RequestBuilder` appropriate
/// for making calls on the Scryfall API
pub struct PlanesWalker(RequestBuilder);

impl PlanesWalker {
    /// main constructor
    fn untap(client: Client, full_url: &str) -> Self {
        Self(
            client
                .get(full_url)
                .header(USER_AGENT, USER_AGENT_VALUE)
                .header(ACCEPT, ACCEPT_VALUE),
        )
    }
    /// for sending requests
    async fn cast(self) -> Result<Response, reqwest::Error> {
        self.0.send().await
    }

    /// adding the "q" parameter with value input
    /// (meant for the tutor function)
    fn tutor_for(self, search_str: &str) -> Self {
        PlanesWalker(self.0.query(&[("q", search_str)]))
    }

    #[allow(dead_code)]
    /// for searching for a single card
    pub async fn tutor(client: Client, search_str: &str) -> anyhow::Result<Vec<ScryfallData>> {
        let url = SCRYFALL_API_BASE.to_string() + CARDS_SEARCH_ENDPOINT;
        let urza = PlanesWalker::untap(client, &url);

        let get_result = urza
            .tutor_for(search_str)
            .cast()
            .await
            .context("failed to get on cards search endpoint")?;
        let get_json = get_result
            .json()
            .await
            .context("failed to parse json from get result")?;
        // tracing::debug!("card response was {:#?}", get_json);
        let card_search_response: ScryfallDataSearchResponse =
            serde_json::from_value(get_json).context("failed to parse CardSearchResponse")?;
        Ok(card_search_response.data)
    }
}

/// for getting a `RequestBuilder` ready with scryfall api url + endpoint
#[allow(dead_code)]
trait CreatePlanesWalker {
    fn into_planeswalker(self, endpoint: &str) -> PlanesWalker;
}

impl CreatePlanesWalker for Client {
    fn into_planeswalker(self, endpoint: &str) -> PlanesWalker {
        PlanesWalker::untap(self, endpoint)
    }
}

// ==========================
//  bulk endpoint ergonomics
// ==========================
// not sure if every endpoints returns consistent types
// only tested `OracleCards` for now :O

pub enum BulkEndpoint {
    OracleCards,
    UniqueArtwork,
    DefaultCards,
    AllCards,
    Rulings,
}

impl BulkEndpoint {
    fn resolve(&self) -> String {
        match self {
            BulkEndpoint::OracleCards => "/bulk-data/oracle-cards".to_string(),
            BulkEndpoint::UniqueArtwork => "/bulk-data/unique-artworks".to_string(),
            BulkEndpoint::DefaultCards => "/bulk-data/default-cards".to_string(),
            BulkEndpoint::AllCards => "/bulk-data/all-cards".to_string(),
            BulkEndpoint::Rulings => "/bulk-data/rulings".to_string(),
        }
    }
}

impl BulkEndpoint {
    /// gets bulk cards with a BulkEndpoint parameter end returns `Vec<ScryfallData>`
    pub async fn amass(&self) -> anyhow::Result<Vec<ScryfallData>> {
        // first get the bulk data object with our main url
        let url = SCRYFALL_API_BASE.to_string() + &self.resolve();
        let urza = PlanesWalker::untap(Client::new(), &url);

        let bulk_response = urza
            .cast()
            .await
            .context("failed to get bulk response with planeswalker")?;

        let bulk_json: Value = bulk_response
            .json()
            .await
            .context("failed to parse json from main uri result")?;

        let bulk_data_object =
            from_value::<BulkDataObject>(bulk_json).context("failed to parse BulkDataObject")?;

        // then use the download_uri to fetch the actual card data
        let karn = PlanesWalker::untap(Client::new(), &bulk_data_object.download_uri);

        let cards_response = karn
            .cast()
            .await
            .context("failed to get download response with planeswalker")?;

        let cards_json: Value = cards_response
            .json()
            .await
            .context("failed to parse json from download uri result")?;

        let cards: Vec<ScryfallData> =
            from_value(cards_json).context("failed to parse Vec<ScryfallData>")?;

        Ok(cards)
    }
}
