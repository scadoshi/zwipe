pub mod was_ago;
use std::{str::FromStr, time::Duration};

use chrono::NaiveDateTime;
use was_ago::WasAgo;

use deck_builder::{
    config::Config,
    domain::card::{self, models::sync_metrics::SyncType, ports::CardService},
    outbound::sqlx::postgres::Postgres,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_env()?;
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::from_str(&config.rust_log)?)
        .init();
    let db = Postgres::new(&config.database_url).await?;
    let service = card::services::Service::new(db);

    loop {
        // get last sync dates
        let last_partial_option: Option<NaiveDateTime> =
            service.get_last_sync_date(SyncType::Partial).await?;

        let last_full_option: Option<NaiveDateTime> =
            service.get_last_sync_date(SyncType::Full).await?;

        // whether to run a full
        if last_full_option.is_none() || last_full_option.unwrap().was_a_month_ago() {
            service.scryfall_sync(SyncType::Full).await?;
        }

        // whether to run a partial
        if (last_partial_option.is_none() || last_partial_option.unwrap().was_a_week_ago())
            && (last_full_option.is_none() || last_full_option.unwrap().was_a_week_ago())
        {
            service.scryfall_sync(SyncType::Partial).await?;
        }

        // only check once an hour
        tokio::time::sleep(Duration::from_secs(3600)).await;
    }
}
