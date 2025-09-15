pub mod was_ago;
use chrono::NaiveDateTime;
use std::{str::FromStr, time::Duration};
use was_ago::WasAgo;
use zwipe::{
    config::Config,
    domain::card::{self, models::sync_metrics::SyncType, ports::CardService},
    domain::logo,
    outbound::sqlx::postgres::Postgres,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logo::print();
    let config = Config::from_env()?;
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::from_str(&config.rust_log)?)
        .init();
    let db = Postgres::new(&config.database_url).await?;
    let service = card::services::Service::new(db);

    tracing::info!("running sync service");
    loop {
        let last_partial: Option<NaiveDateTime> =
            service.get_last_sync_date(SyncType::Partial).await?;

        let last_full: Option<NaiveDateTime> = service.get_last_sync_date(SyncType::Full).await?;

        if last_full.is_none() || last_full.unwrap().was_a_month_ago() {
            let _sync_metrics = service.scryfall_sync(SyncType::Full).await?;
        }

        if (last_partial.is_none() || last_partial.unwrap().was_a_week_ago())
            && (last_full.is_none() || last_full.unwrap().was_a_week_ago())
        {
            let _sync_metrics = service.scryfall_sync(SyncType::Partial).await?;
        }

        let one_hour: Duration = Duration::from_secs(3600);
        tokio::time::sleep(one_hour).await;
    }
}
