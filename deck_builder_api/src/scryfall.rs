use reqwest::{
    header::{ACCEPT, USER_AGENT},
    Response,
};
use serde::Deserialize;
use std::io::Write;

use crate::models::card::scryfall_card::ScryfallCard;

// this generates a new client every time
// very slow
// update later
trait Scryfall {
    async fn scry(&self) -> Result<Response, reqwest::Error>;
}

impl Scryfall for String {
    async fn scry(&self) -> Result<Response, reqwest::Error> {
        reqwest::Client::new()
            .get(self)
            .header(USER_AGENT, "DeckBuilderAPI/0.0")
            .header(ACCEPT, "*/*")
            .send()
            .await
    }
}

#[derive(Deserialize, Debug)]
struct CardSearchResponse {
    data: Vec<ScryfallCard>,
    has_more: bool,
    object: String,
    total_cards: i32,
}

pub async fn card_search(
    search_str: &str,
) -> Result<Vec<ScryfallCard>, Box<dyn std::error::Error>> {
    let scryfall_card_search_url = "https://api.scryfall.com/cards/search?q=";

    let full_url = scryfall_card_search_url.to_string() + &urlencoding::encode(search_str);

    let response = full_url.scry().await?;
    let json = response.json().await?;
    let card_search_response: CardSearchResponse = serde_json::from_value(json)?;

    let mut output = std::fs::File::create("../dump/card_search_result.rs")?;
    output.write(format!("{:#?}", card_search_response).as_ref())?;

    println!("(*3*)<(wrote to card_search_result.json!)");

    Ok(card_search_response.data)
}

#[derive(Deserialize, Debug)]
struct BulkDataObject {
    content_encoding: String,
    content_type: String,
    description: String,
    download_uri: String,
    id: String,
    name: String,
    object: String,
    size: i64,
    #[serde(rename = "type")]
    bulk_type: String,
    updated_at: String,
    uri: String,
}

pub async fn get_oracle_card_dump() -> Result<(), Box<dyn std::error::Error>> {
    let bulk_data_object_url = "https://api.scryfall.com/bulk-data/oracle-cards".to_string();
    let response = bulk_data_object_url.scry().await?;
    let json = response.json().await?;
    let bulk_data_object: BulkDataObject = serde_json::from_value(json)?;

    let download_response = bulk_data_object.download_uri.scry().await?;
    let download_json = download_response.json().await?;

    let mut output = std::fs::File::create("../dump/dump.json")?;

    output.write(format!("{:#?}", download_json).as_ref())?;

    println!("(*3*)<(wrote to dump.json!)");

    Ok(())
}
