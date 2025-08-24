use chrono::NaiveDateTime;

use crate::{
    domain::card::{
        models::{
            scryfall_card::ScryfallCard,
            sync_metrics::{SyncMetrics, SyncType},
            GetCardError, SearchCardError,
        },
        ports::{CardRepository, CardService},
    },
    inbound::http::scryfall::BulkEndpoint,
};

#[derive(Debug, Clone)]
pub struct Service<R>
where
    R: CardRepository,
{
    repo: R,
}

impl<R> Service<R>
where
    R: CardRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

impl<R: CardRepository> CardService for Service<R> {
    async fn get_card(&self, id: &uuid::Uuid) -> Result<ScryfallCard, GetCardError> {
        todo!()
    }

    async fn search_cards(
        &self,
        params: super::models::CardSearchParameters,
    ) -> Result<Vec<ScryfallCard>, SearchCardError> {
        todo!()
    }

    async fn scryfall_sync(&self, sync_type: SyncType) -> anyhow::Result<()> {
        let mut sync_metrics = SyncMetrics::new(sync_type.clone());

        // just going to hard code these for now
        let batch_size = 500;
        let bulk_endpoint = BulkEndpoint::OracleCards;
        let cards = bulk_endpoint.download().await?;

        sync_metrics.set_total_cards_count(cards.len() as i32);

        match sync_type {
            SyncType::Full => {
                self.repo
                    .delete_if_exists_and_batch_insert(cards, batch_size, &mut sync_metrics)
                    .await?;
            }
            SyncType::Partial => {
                self.repo
                    .batch_insert_if_not_exists(cards, batch_size, &mut sync_metrics)
                    .await?;
            }
        };

        sync_metrics.mark_as_completed();

        self.repo.record_sync_metrics(sync_metrics).await?;

        Ok(())
    }

    async fn get_last_sync_date(
        &self,
        sync_type: SyncType,
    ) -> anyhow::Result<Option<NaiveDateTime>> {
        self.repo.get_last_sync_date(sync_type).await
    }
}
