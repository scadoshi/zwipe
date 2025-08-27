// internal
use zwipe::{
    config::Config,
    domain::card::{self, models::scryfall_card::ScryfallCard, ports::CardService},
    domain::logo,
    inbound::http::scryfall::PlanesWalker,
    outbound::sqlx::postgres::Postgres,
};
// external
use anyhow::Context;
use reqwest::Client;
use std::str::FromStr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logo::print();
    let config = Config::from_env()?;
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::from_str(&config.rust_log)?)
        .init();
    tracing::info!("running sync test");
    let db = Postgres::new(&config.database_url).await?;
    tracing::info!("database connection established");
    let service = card::services::Service::new(db);
    let card_search_string = "starscream";
    tracing::info!("searching for {:?}", card_search_string);
    let card = PlanesWalker::tutor(&mut Client::new(), card_search_string)
        .await?
        .get(0)
        .context("failed to get card")?
        .clone();
    tracing::info!("found {:?}", card.name);
    let _inserted_card: ScryfallCard = service.insert(card).await?;
    tracing::info!("successfully inserted into database");
    Ok(())
}
