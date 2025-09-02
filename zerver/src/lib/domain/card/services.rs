use crate::domain::card::{
    models::{
        card_profile::{
            CardProfile, GetCardProfileError, GetCardProfileRequest, GetCardProfilesRequest,
        },
        scryfall_data::{
            CreateScryfallDataError, GetScryfallDataError, GetScryfallDataRequest,
            GetScryfallDatasRequest, ScryfallData, SearchScryfallDataError,
            SearchScryfallDataRequest,
        },
        sync_metrics::{SyncMetrics, SyncType},
    },
    ports::{CardRepository, CardService},
};
use crate::inbound::http::scryfall::BulkEndpoint;
use crate::outbound::sqlx::card::scryfall_data_field_count;
use chrono::NaiveDateTime;

/// postgresql will have issues if there are more
/// parameters than this in any single query
const POSTGRESQL_PARAMETER_HARD_LIMIT: usize = 65_535;

/// calculates batch size based on limit
/// based on the number of fields that `ScryfallData` has
///
/// limits to half of maximum to keep queries running quickly
fn batch_size() -> usize {
    POSTGRESQL_PARAMETER_HARD_LIMIT / 2 / scryfall_data_field_count()
}

/// structure which implements `CardService`
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
    async fn insert(&self, card: ScryfallData) -> Result<ScryfallData, CreateScryfallDataError> {
        self.repo.insert(card).await
    }

    async fn get_card(
        &self,
        request: &GetScryfallDataRequest,
    ) -> Result<ScryfallData, GetScryfallDataError> {
        self.repo.get_card(request).await
    }

    async fn get_cards(
        &self,
        request: &GetScryfallDatasRequest,
    ) -> Result<Vec<ScryfallData>, GetScryfallDataError> {
        self.repo.get_cards(request).await
    }

    async fn search_scryfall_datas(
        &self,
        request: &SearchScryfallDataRequest,
    ) -> Result<Vec<ScryfallData>, SearchScryfallDataError> {
        self.repo.search_cards(request).await
    }

    async fn get_card_profile(
        &self,
        request: &GetCardProfileRequest,
    ) -> Result<CardProfile, GetCardProfileError> {
        self.repo.get_card_profile(request).await
    }

    async fn get_card_profiles(
        &self,
        request: &GetCardProfilesRequest,
    ) -> Result<Vec<CardProfile>, GetCardProfileError> {
        self.repo.get_card_profiles(request).await
    }

    async fn scryfall_sync(&self, sync_type: SyncType) -> anyhow::Result<SyncMetrics> {
        let mut sync_metrics = SyncMetrics::generate(sync_type.clone());

        let batch_size = batch_size();

        // just going to hard code this for now
        let bulk_endpoint = BulkEndpoint::OracleCards;
        let cards = bulk_endpoint.amass().await?;

        sync_metrics.set_received(cards.len() as i32);

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

        let sync_metrics = self.repo.record_sync_metrics(sync_metrics).await?;

        tracing::info!("{:?}", sync_metrics);

        Ok(sync_metrics)
    }

    async fn get_last_sync_date(
        &self,
        sync_type: SyncType,
    ) -> anyhow::Result<Option<NaiveDateTime>> {
        self.repo.get_last_sync_date(sync_type).await
    }
}
