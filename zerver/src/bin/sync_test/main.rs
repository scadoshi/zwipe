// internal
use zwipe::{
    config::Config,
    domain::card::{self, models::scryfall_card::ScryfallCard, ports::CardService},
    inbound::http::scryfall::PlanesWalker,
    outbound::sqlx::postgres::Postgres,
};
// external
use anyhow::Context;
use reqwest::Client;
use std::str::FromStr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_env()?;
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::from_str(&config.rust_log)?)
        .init();
    let db = Postgres::new(&config.database_url).await?;
    let service = card::services::Service::new(db);
    let card = PlanesWalker::tutor(&mut Client::new(), "starscream")
        .await?
        .get(0)
        .context("failed to get card")?
        .clone();
    let _inserted_card: ScryfallCard = service.insert(card).await?;

    tracing::info!("\n\nsuccess!\n\n");

    Ok(())
}
