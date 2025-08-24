use std::fmt::Display;

use anyhow::Context;
use reqwest::{
    header::{ACCEPT, USER_AGENT},
    Client, Response,
};
use serde::Deserialize;
use serde_json::from_value;

use crate::domain::card::models::scryfall_card::ScryfallCard;

// ==================================================
//       to use in scryfall requests
// ==================================================
const USER_AGENT_VALUE: &str = "DeckBuilderAPI/0.0";
const ACCEPT_VALUE: &str = "*/*";
const SCRYFALL_API_BASE: &str = "https://api.scryfall.com";

pub enum BulkEndpoint {
    OracleCards,
    UniqueArtwork,
    DefaultCards,
    AllCards,
    Rulings,
}

impl Display for BulkEndpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BulkEndpoint::OracleCards => write!(f, "/bulk-data/oracle-cards"),
            BulkEndpoint::UniqueArtwork => write!(f, "/bulk-data/unique-artworks"),
            BulkEndpoint::DefaultCards => write!(f, "/bulk-data/default-cards"),
            BulkEndpoint::AllCards => write!(f, "/bulk-data/all-cards"),
            BulkEndpoint::Rulings => write!(f, "/bulk-data/rulings"),
        }
    }
}

impl BulkEndpoint {
    fn full_url(&self) -> String {
        SCRYFALL_API_BASE.to_string() + self.to_string().as_str()
    }
}

// ================================
//          helper functions
// ================================
trait Scry {
    async fn scry(&self, client: &Client) -> Result<Response, reqwest::Error>;
}
impl Scry for &str {
    async fn scry(&self, client: &Client) -> Result<Response, reqwest::Error> {
        client
            .get(*self)
            .header(USER_AGENT, USER_AGENT_VALUE)
            .header(ACCEPT, ACCEPT_VALUE)
            .send()
            .await
    }
}

impl Scry for String {
    async fn scry(&self, client: &Client) -> Result<Response, reqwest::Error> {
        <&str as Scry>::scry(&self.as_str(), client).await
    }
}

// =================================================
//  this is old but might use later
// =================================================

// #[derive(Deserialize, Debug)]
// struct CardSearchResponse {
//     data: Vec<ScryfallCard>,
//     has_more: bool,
//     object: String,
//     total_cards: i32,
// }

// pub async fn card_search(search_str: &str) -> Result<Vec<ScryfallCard>, Box<dyn StdError>> {
//     Ok(from_value::<CardSearchResponse>(
//         ("https://api.scryfall.com/cards/search?q=".to_string() + &urlencoding::encode(search_str))
//             .scry()
//             .await?
//             .json()
//             .await?,
//     )?
//     .data)
// }

// ============================================
/// when you hit Scryfall's bulk data endpoint
/// you get this instead of the cards
/// download the cards with the download uri
// ============================================

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

/// Gets bulk cards with a BulkEndpoint parameter end returns Vec<ScryfallCard>
pub async fn get_bulk(bulk_endpoint: BulkEndpoint) -> anyhow::Result<Vec<ScryfallCard>> {
    let full_url: String = bulk_endpoint.full_url();
    let client = Client::new();

    // First, fetch the bulk data object from the endpoint
    let bulk_response = full_url
        .scry(&client)
        .await
        .context(format!("failed to scry full_url {}", full_url))?;

    let bulk_json = bulk_response
        .json()
        .await
        .context("failed to parse json from full_url result")?;

    let bulk_data_object =
        from_value::<BulkDataObject>(bulk_json).context("failed to parse BulkDataObject")?;

    // Then, use the download_uri to fetch the actual card data
    let cards_response = bulk_data_object
        .download_uri
        .scry(&client)
        .await
        .context("failed to scry download uri from BulkDataObject")?;

    let cards_json = cards_response
        .json()
        .await
        .context("failed to parse json from download uri result")?;

    let cards =
        from_value::<Vec<ScryfallCard>>(cards_json).context("failed to parse Vec<ScryfallCard>")?;

    Ok(cards)
}
