use reqwest::header::{ACCEPT, USER_AGENT};
use serde::Deserialize;

use crate::models::card::{Card, NewCard};
use std::{error::Error as StdError, io::Write};

#[derive(Deserialize, Debug)]
struct CardSearchResponse {
    data: Vec<NewCard>,
    has_more: bool,
    object: String,
    total_cards: i32,
}

pub async fn scryfall_card_search(search_str: &str) -> Result<Vec<Card>, Box<dyn StdError>> {
    let scryfall_card_search_url = "https://api.scryfall.com/cards/search?q=";
    let url_encoded_search_str = urlencoding::encode(search_str);
    let full_url = scryfall_card_search_url.to_string() + &url_encoded_search_str;

    let result = reqwest::Client::new()
        .get(full_url)
        .header(USER_AGENT, "DeckBuilderAPI/0.1")
        .header(ACCEPT, "*/*")
        .send()
        .await?;

    let body: serde_json::Value = result.json().await?;
    let mut output = std::fs::File::create("../card_search_result.json")?;
    output.write(serde_json::to_string_pretty(&body)?.as_ref())?;
    println!("\n(*3*)<(wrote output to a file! go find it!)\n");

    Ok(Vec::new())
}
