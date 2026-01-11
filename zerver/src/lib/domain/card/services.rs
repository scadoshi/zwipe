use crate::domain::card::models::get_artists::GetArtistsError;
use crate::domain::card::models::get_card_types::GetCardTypesError;
use crate::domain::card::models::get_sets::GetSetsError;
use crate::domain::card::models::scryfall_data::get_scryfall_data::{
    GetScryfallData, ScryfallDataIds,
};
use crate::inbound::external::scryfall::bulk::BulkEndpoint;
use crate::{
    domain::card::{
        models::{
            card_profile::{
                get_card_profile::{CardProfileIds, GetCardProfile, GetCardProfileError},
                CardProfile,
            },
            create_card::CreateCardError,
            get_card::GetCardError,
            scryfall_data::ScryfallData,
            search_card::{card_filter::CardFilter, error::SearchCardsError},
            sync_metrics::SyncMetrics,
            Card,
        },
        ports::{CardRepository, CardService},
    },
    outbound::sqlx::card::helpers::scryfall_data_fields::scryfall_data_field_count,
};
use chrono::NaiveDateTime;

/// postgresql will have issues if there are more parameters than this in any single query
const POSTGRESQL_PARAMETER_HARD_LIMIT: usize = 65_535;

/// calculates batch size based on limit based on the number of fields that `ScryfallData` has
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
    // ========
    //  create
    // ========
    async fn upsert(&self, scryfall_data: ScryfallData) -> Result<Card, CreateCardError> {
        self.repo.upsert(&scryfall_data).await
    }

    async fn scryfall_sync(&self, bulk_endpoint: BulkEndpoint) -> anyhow::Result<SyncMetrics> {
        tracing::info!(
            "performing scryfall sync with {}",
            bulk_endpoint.to_snake_case()
        );
        let mut sync_metrics = SyncMetrics::new();
        let batch_size = batch_size();
        // just going to hard code this for now
        let scryfall_data = bulk_endpoint.amass(false).await?;
        sync_metrics.set_received_count(scryfall_data.len() as i32);
        self.repo
            .batch_delta_upsert(&scryfall_data, batch_size, &mut sync_metrics)
            .await?;
        sync_metrics.mark_as_completed();
        let sync_metrics = self.repo.record_sync_metrics(&sync_metrics).await?;
        tracing::info!("{:?}", sync_metrics);
        Ok(sync_metrics)
    }

    // =====
    //  get
    // =====
    async fn get_card(&self, request: &GetScryfallData) -> Result<Card, GetCardError> {
        self.repo.get_card(request).await
    }

    async fn get_cards(&self, request: &ScryfallDataIds) -> Result<Vec<Card>, GetCardError> {
        self.repo.get_cards(request).await
    }

    async fn search_cards(&self, request: &CardFilter) -> Result<Vec<Card>, SearchCardsError> {
        self.repo.search_cards(request).await
    }

    async fn get_artists(&self) -> Result<Vec<String>, GetArtistsError> {
        self.repo.get_artists().await
    }

    async fn get_card_types(&self) -> Result<Vec<String>, GetCardTypesError> {
        self.repo.get_card_types().await
    }

    async fn get_sets(&self) -> Result<Vec<String>, GetSetsError> {
        self.repo.get_sets().await
    }

    async fn get_card_profile_with_id(
        &self,
        request: &GetCardProfile,
    ) -> Result<CardProfile, GetCardProfileError> {
        self.repo.get_card_profile_with_id(request).await
    }

    async fn get_card_profile_with_scryfall_data_id(
        &self,
        request: &GetScryfallData,
    ) -> Result<CardProfile, GetCardProfileError> {
        self.repo
            .get_card_profile_with_scryfall_data_id(request)
            .await
    }

    async fn get_card_profiles_with_ids(
        &self,
        request: &CardProfileIds,
    ) -> Result<Vec<CardProfile>, GetCardProfileError> {
        self.repo.get_card_profiles_with_ids(request).await
    }

    async fn get_card_profiles_with_scryfall_data_ids(
        &self,
        request: &ScryfallDataIds,
    ) -> Result<Vec<CardProfile>, GetCardProfileError> {
        self.repo
            .get_card_profiles_with_scryfall_data_ids(request)
            .await
    }

    async fn get_last_sync_date(&self) -> anyhow::Result<Option<NaiveDateTime>> {
        self.repo.get_last_sync_date().await
    }
}
