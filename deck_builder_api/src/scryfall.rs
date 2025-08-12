use reqwest::{
    header::{ACCEPT, USER_AGENT},
    Response,
};
use serde::Deserialize;
use serde_json::from_value;
use std::error::Error as StdError;

use crate::models::card::scryfall_card::ScryfallCard;

// this generates a new client every time
// very slow
// update later
trait Scry {
    async fn scry(&self) -> Result<Response, reqwest::Error>;
}
impl Scry for &str {
    async fn scry(&self) -> Result<Response, reqwest::Error> {
        reqwest::Client::new()
            .get(*self)
            .header(USER_AGENT, "DeckBuilderAPI/0.0")
            .header(ACCEPT, "*/*")
            .send()
            .await
    }
}

impl Scry for String {
    async fn scry(&self) -> Result<Response, reqwest::Error> {
        <&str as Scry>::scry(&self.as_str()).await
    }
}

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

pub async fn get_oracle_card_dump() -> Result<Vec<ScryfallCard>, Box<dyn StdError>> {
    Ok(from_value::<Vec<ScryfallCard>>(
        from_value::<BulkDataObject>(
            "https://api.scryfall.com/bulk-data/oracle-cards"
                .scry()
                .await?
                .json()
                .await?,
        )?
        .download_uri
        .scry()
        .await?
        .json()
        .await?,
    )?)
}
