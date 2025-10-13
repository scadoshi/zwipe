use anyhow::Context;
use reqwest::Client;
use zwipe::{
    config::Config,
    domain::{
        card::{self, models::Card, ports::CardService},
        logo,
    },
    inbound::external::scryfall::PlanesWalker,
    outbound::sqlx::postgres::Postgres,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logo::print();
    let config = Config::from_env()?;
    tracing_subscriber::fmt()
        .with_max_level(config.rust_log)
        .init();
    tracing::info!("running sync test");
    let db = Postgres::new(&config.database_url).await?;
    tracing::info!("database connection established");
    let service = card::services::Service::new(db);
    let card_search_string = "satya";
    tracing::info!("searching for {:?}", card_search_string);
    let card = PlanesWalker::tutor(&mut Client::new(), card_search_string)
        .await?
        .get(0)
        .context("failed to get card")?
        .clone();
    tracing::info!("found {:?}", card.name);
    let _inserted_card: Card = service.insert(card).await?;
    tracing::info!("successfully inserted into database");
    Ok(())
}
