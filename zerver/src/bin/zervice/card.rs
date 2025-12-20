use chrono::{Duration, Utc};
use std::future::Future;
use zwipe::domain::card::{
    self,
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
        if self
            .get_last_sync_date()
            .await?
            .is_none_or(|d| d < Utc::now().naive_utc() - Duration::days(7))
        {
            self.scryfall_sync().await?;
        }
        Ok(())
    }
}
