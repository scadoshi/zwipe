use crate::was_ago::WasAgo;
use chrono::NaiveDateTime;
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
        let last_sync: Option<NaiveDateTime> = self.get_last_sync_date().await?;
        if last_sync.is_none_or(|d| d.was_a_week_ago()) {
            self.scryfall_sync().await?;
        }
        Ok(())
    }
}
