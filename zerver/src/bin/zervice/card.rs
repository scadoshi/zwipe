use std::future::Future;

use crate::was_ago::WasAgo;
use chrono::NaiveDateTime;
use zwipe::domain::card::{
    self,
    models::sync_metrics::SyncType,
    ports::{CardRepository, CardService},
};

pub trait CheckCards {
    fn check_cards(&self) -> impl Future<Output = anyhow::Result<()>> + Send;
}

impl<R> CheckCards for card::services::Service<R>
where
    R: CardRepository,
{
    async fn check_cards(&self) -> anyhow::Result<()> {
        let last_partial: Option<NaiveDateTime> =
            self.get_last_sync_date(SyncType::Partial).await?;

        let last_full: Option<NaiveDateTime> = self.get_last_sync_date(SyncType::Full).await?;

        if last_full.map_or(true, |d| d.was_a_month_ago()) {
            self.scryfall_sync(SyncType::Full).await?;
        }

        if (last_partial.map_or(true, |d| d.was_a_week_ago()))
            && (last_full.map_or(true, |d| d.was_a_week_ago()))
        {
            self.scryfall_sync(SyncType::Partial).await?;
        }

        Ok(())
    }
}
