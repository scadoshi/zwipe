use reqwest::{
    header::{ACCEPT, USER_AGENT},
    Response,
};
use serde::Deserialize;
use serde_json::from_value;
use sqlx::{query_scalar, PgPool};
use std::error::Error as StdError;
use tracing::info;

use crate::{
    domain::models::scryfall_card::ScryfallCard,
    outbound::database::card::{delete_all, MultipleInsert},
};

// this generates a new client every time
// very slow update at some point
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

pub async fn fetch_oracle_cards() -> Result<Vec<ScryfallCard>, Box<dyn StdError>> {
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

pub async fn scryfall_sync(pg_pool: &PgPool) -> Result<(), Box<dyn StdError>> {
    delete_all(&pg_pool).await?;
    info!("Deleted all cards ");
    info!("Beginning Scryfall sync");
    info!("Fetching oracle cards");
    let bulk_data: Vec<ScryfallCard> = fetch_oracle_cards().await?;
    info!("Scryfall returned {} cards", bulk_data.len());
    let scryfall_cards_row_count: i64 = query_scalar("SELECT COUNT(id) FROM scryfall_cards")
        .fetch_one(pg_pool)
        .await?;
    info!("Database has {} cards", scryfall_cards_row_count);
    let batch_size = 500;
    info!("Smart inserting by batch size {:?}", batch_size);
    bulk_data.smart_insert(batch_size, &pg_pool).await?;
    let scryfall_cards_row_count: i64 = query_scalar("SELECT COUNT(id) FROM scryfall_cards")
        .fetch_one(pg_pool)
        .await?;
    info!("Database now has {} cards", scryfall_cards_row_count);

    info!("Database sync completed");
    Ok(())
}
