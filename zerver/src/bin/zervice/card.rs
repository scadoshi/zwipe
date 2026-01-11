use chrono::{Duration, Utc};
use std::future::Future;
use zwipe::{
    domain::card::{
        self,
        ports::{CardRepository, CardService},
    },
    inbound::external::scryfall::bulk::BulkEndpoint,
};

pub trait CheckCardsAgainst {
    fn check_cards_against(
        &self,
        bulk_endpoint: BulkEndpoint,
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
}

impl<R> CheckCardsAgainst for card::services::Service<R>
where
    R: CardRepository,
{
    async fn check_cards_against(&self, bulk_endpoint: BulkEndpoint) -> anyhow::Result<()> {
        if self
            .get_last_sync_date()
            .await?
            .is_none_or(|d| d < Utc::now().naive_utc() - Duration::days(7))
        {
            self.scryfall_sync(bulk_endpoint).await?;
        }
        Ok(())
    }
}
