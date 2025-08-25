use std::str::FromStr;

use anyhow::Context;
use deck_builder::{
    config::Config,
    domain::card::{self, models::scryfall_card::ScryfallCard, ports::CardService},
    inbound::http::scryfall::PlanesWalker,
    outbound::sqlx::postgres::Postgres,
};
use reqwest::Client;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_env()?;
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::from_str(&config.rust_log)?)
        .init();
    let db = Postgres::new(&config.database_url).await?;
    let service = card::services::Service::new(db);

    let card = PlanesWalker::tutor(&mut Client::new(), "sonic the hedgehog")
        .await?
        .get(0)
        .context("failed to get card")?
        .clone();

    let card: ScryfallCard = service.insert_with_card_response(card).await?;

    tracing::info!("success!");

    Ok(())
}
