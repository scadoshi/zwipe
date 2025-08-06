use reqwest::header::{ACCEPT, USER_AGENT};
use serde::Deserialize;

use crate::models::card::NewCard;
use std::{error::Error as StdError, io::Write};

#[derive(Deserialize, Debug)]
struct CardSearchResponse {
    data: Vec<NewCard>,
    has_more: bool,
    object: String,
    total_cards: i32,
}

pub async fn card_search(search_str: &str) -> Result<Vec<NewCard>, Box<dyn StdError>> {
    let scryfall_card_search_url = "https://api.scryfall.com/cards/search?q=";
    let full_url = scryfall_card_search_url.to_string() + &urlencoding::encode(search_str);

    let result = reqwest::Client::new()
        .get(full_url)
        .header(USER_AGENT, "DeckBuilderAPI/0.1")
        .header(ACCEPT, "*/*")
        .send()
        .await?;

    let body: serde_json::Value = result.json().await?;
    let card_search_response: CardSearchResponse = serde_json::from_value(body)?;

    let mut output = std::fs::File::create("../card_search_result.rs")?;
    output.write(format!("{:#?}", card_search_response.data).as_ref())?;
    // output.write(serde_json::to_string_pretty(&body)?.as_ref())?;

    println!("\n(*3*)<(wrote output to a file! go find it!)\n");

    Ok(Vec::new())
}
