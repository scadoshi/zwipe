use crate::inbound::external::scryfall::bulk::BulkEndpoint;
use zwipe_core::domain::card::{
    Card,
    card_profile::CardProfile,
    mechanical_category::classify_by_heuristics,
    scryfall_data::ScryfallData,
    search_card::card_filter::CardFilter,
};
use crate::{
    domain::card::{
        models::{
            search_card::error::SearchCardsError,
            zervice_metrics::ZerviceMetrics,
        },
        ports::{CardRepository, CardService},
        requests::{
            create_card::CreateCardError,
            get_artists::GetArtistsError,
            get_card::GetCardError,
            get_card_profile::{CardProfileIds, GetCardProfile, GetCardProfileError},
            get_card_types::GetCardTypesError,
            get_keywords::GetKeywordsError,
            get_languages::GetLanguagesError,
            get_oracle_words::GetOracleWordsError,
            get_scryfall_data::{GetScryfallData, ScryfallDataIds},
            get_sets::GetSetsError,
        },
    },
    outbound::sqlx::card::helpers::scryfall_data_fields::scryfall_data_field_count,
};
use chrono::NaiveDateTime;

/// PostgreSQL parameter limit per query (~65k parameters).
///
/// Exceeding this limit causes "prepared statement contains too many parameters" errors.
const POSTGRESQL_PARAMETER_HARD_LIMIT: usize = 65_535;

/// Calculates optimal batch size for bulk card upserts.
///
/// Determines how many cards can be upserted in a single query without exceeding
/// PostgreSQL's parameter limit. Uses half the limit for performance.
///
/// # Formula
/// `batch_size = (POSTGRESQL_PARAM_LIMIT / 2) / scryfall_data_field_count()`
///
/// With ~100 fields per card and 65k param limit, this yields ~327 cards per batch.
fn batch_size() -> usize {
    POSTGRESQL_PARAMETER_HARD_LIMIT / 2 / scryfall_data_field_count()
}

/// Card service implementation handling MTG card operations and Scryfall synchronization.
///
/// This service coordinates:
/// - **Scryfall sync**: Bulk downloads and delta upserts with batch processing
/// - **Card queries**: Search, get by ID, filter by various criteria
/// - **Metadata queries**: Artists, card types, sets, languages
/// - **Card profiles**: Internal metadata tracking (sync timestamps, DB IDs)
///
/// # Performance
/// Bulk operations use batch processing to avoid PostgreSQL parameter limits.
/// Batch size is auto-calculated based on ScryfallData field count (~327 cards/batch).
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
    /// Creates a new card service with the provided repository.
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

    async fn scryfall_sync(&self, bulk_endpoint: BulkEndpoint) -> anyhow::Result<ZerviceMetrics> {
        tracing::info!(
            "performing scryfall sync with {}",
            bulk_endpoint.to_snake_case()
        );
        let mut zervice_metrics = ZerviceMetrics::new();
        let batch_size = batch_size();
        let scryfall_data: Vec<ScryfallData> = bulk_endpoint.amass().await?;
        zervice_metrics.set_received_count(scryfall_data.len() as i32);
        self.repo
            .batch_delta_upsert(&scryfall_data, batch_size, &mut zervice_metrics)
            .await?;
        zervice_metrics.mark_as_completed();
        let zervice_metrics = self.repo.record_zervice_metrics(&zervice_metrics).await?;
        tracing::info!("sync complete: {}", zervice_metrics);
        Ok(zervice_metrics)
    }

    async fn classify_untagged_cards(&self, batch_size: usize) -> anyhow::Result<(u32, u32)> {
        let all_ids = self.repo.get_unclassified_card_ids().await?;
        let total = all_ids.len() as u32;
        if total == 0 {
            return Ok((0, 0));
        }

        let mut classified = 0u32;

        for chunk in all_ids.chunks(batch_size) {
            let cards = self.repo.get_cards_batch(chunk).await?;

            let updates: Vec<_> = cards
                .iter()
                .map(|card| {
                    let cats = classify_by_heuristics(card);
                    (card.scryfall_data.id, cats)
                })
                .filter(|(_, cats)| !cats.is_empty())
                .collect();

            classified += updates.len() as u32;
            if !updates.is_empty() {
                self.repo.update_mechanical_categories(&updates).await?;
            }
        }

        Ok((classified, total))
    }

    async fn clear_all_categories(&self) -> anyhow::Result<()> {
        self.repo.clear_all_categories().await
    }

    async fn refresh_latest_cards(&self) -> anyhow::Result<()> {
        self.repo.refresh_latest_cards().await
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

    async fn get_printings(&self, oracle_id: uuid::Uuid) -> Result<Vec<Card>, GetCardError> {
        self.repo.get_printings(oracle_id).await
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

    async fn get_keywords(&self) -> Result<Vec<String>, GetKeywordsError> {
        self.repo.get_keywords().await
    }

    async fn get_oracle_words(&self) -> Result<Vec<String>, GetOracleWordsError> {
        self.repo.get_oracle_words().await
    }

    async fn get_sets(&self) -> Result<Vec<String>, GetSetsError> {
        self.repo.get_sets().await
    }

    async fn get_languages(&self) -> Result<Vec<String>, GetLanguagesError> {
        self.repo.get_languages().await
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

    async fn find_cards_by_exact_names(
        &self,
        names: &[String],
    ) -> Result<Vec<Card>, SearchCardsError> {
        self.repo.find_cards_by_exact_names(names).await
    }
}
