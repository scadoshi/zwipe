use std::future::Future;

use chrono::NaiveDateTime;

use crate::{
    domain::card::models::{
        card_profile::{
            get_card_profile::{CardProfileIds, GetCardProfile, GetCardProfileError},
            CardProfile,
        },
        create_card::CreateCardError,
        get_artists::GetArtistsError,
        get_card::GetCardError,
        get_card_types::GetCardTypesError,
        get_languages::GetLanguagesError,
        get_sets::GetSetsError,
        scryfall_data::{
            get_scryfall_data::{
                GetScryfallData, GetScryfallDataError, ScryfallDataIds, SearchScryfallDataError,
            },
            ScryfallData,
        },
        search_card::{card_filter::CardFilter, error::SearchCardsError},
        sync_metrics::SyncMetrics,
        Card,
    },
    inbound::external::scryfall::bulk::BulkEndpoint,
};

/// enables card related database operations
pub trait CardRepository: Clone + Send + Sync + 'static {
    // ========
    //  create
    // ========

    /// single insert/update of `ScryfallData`
    /// - also inserts/updates `CardProfile`
    fn upsert(
        &self,
        scryfall_data: &ScryfallData,
    ) -> impl Future<Output = Result<Card, CreateCardError>> + Send;

    /// bulk insert/update of `ScryfallData`
    /// - also inserts/updates `CardProfile`
    /// - includes no special batching
    /// - beware of `PostgreSQL` parameter limits
    fn bulk_upsert(
        &self,
        multiple_scryfall_data: &[ScryfallData],
    ) -> impl Future<Output = Result<Vec<Card>, CreateCardError>> + Send;

    /// batch insert/update of `ScryfallData`
    /// - also inserts/updates `CardProfile`
    /// - uses `BulkUpsertWithTx` interally to perform batching
    fn batch_upsert(
        &self,
        multiple_scryfall_data: &[ScryfallData],
        batch_size: usize,
        sync_metrics: &mut SyncMetrics,
    ) -> impl Future<Output = Result<Vec<Card>, CreateCardError>> + Send;

    /// inserts/updates `ScryfallData` only when newer than existing database records
    /// - also inserts/updates `CardProfile`
    /// - skips records that are already up to date
    fn batch_delta_upsert(
        &self,
        multiple_scryfall_data: &[ScryfallData],
        batch_size: usize,
        sync_metrics: &mut SyncMetrics,
    ) -> impl Future<Output = Result<Vec<Card>, CreateCardError>> + Send;

    /// saves `SyncMetrics` to database
    fn record_sync_metrics(
        &self,
        sync_metrics: &SyncMetrics,
    ) -> impl Future<Output = Result<SyncMetrics, anyhow::Error>> + Send;

    // =====
    //  get
    // =====

    /// gets scryfall data with a scryfall data id
    fn get_scryfall_data(
        &self,
        request: &GetScryfallData,
    ) -> impl Future<Output = Result<ScryfallData, GetScryfallDataError>> + Send;

    /// gets multiple scryfall data with a list of scryfall data ids
    fn get_multiple_scryfall_data(
        &self,
        request: &ScryfallDataIds,
    ) -> impl Future<Output = Result<Vec<ScryfallData>, GetScryfallDataError>> + Send;

    /// search for scryfall data given parameters
    fn search_scryfall_data(
        &self,
        request: &CardFilter,
    ) -> impl Future<Output = Result<Vec<ScryfallData>, SearchScryfallDataError>> + Send;

    /// gets card with a scryfall data id
    fn get_card(
        &self,
        request: &GetScryfallData,
    ) -> impl Future<Output = Result<Card, GetCardError>> + Send;

    /// gets cards with a list of scryfall data ids
    fn get_cards(
        &self,
        request: &ScryfallDataIds,
    ) -> impl Future<Output = Result<Vec<Card>, GetCardError>> + Send;

    /// search for cards given parameters
    fn search_cards(
        &self,
        request: &CardFilter,
    ) -> impl Future<Output = Result<Vec<Card>, SearchCardsError>> + Send;

    /// gets all distinct artists from cards
    fn get_artists(&self) -> impl Future<Output = Result<Vec<String>, GetArtistsError>> + Send;

    /// gets all distinct types from cards
    fn get_card_types(&self)
        -> impl Future<Output = Result<Vec<String>, GetCardTypesError>> + Send;

    /// gets all distinct sets form cards
    fn get_sets(&self) -> impl Future<Output = Result<Vec<String>, GetSetsError>> + Send;

    /// gets all distinct languages from cards
    fn get_languages(&self) -> impl Future<Output = Result<Vec<String>, GetLanguagesError>> + Send;

    /// gets card profile with its uuid
    fn get_card_profile_with_id(
        &self,
        request: &GetCardProfile,
    ) -> impl Future<Output = Result<CardProfile, GetCardProfileError>> + Send;

    /// gets card profile with a scryfall data id
    fn get_card_profile_with_scryfall_data_id(
        &self,
        request: &GetScryfallData,
    ) -> impl Future<Output = Result<CardProfile, GetCardProfileError>> + Send;

    /// gets card profile with a list of its uuid
    fn get_card_profiles_with_ids(
        &self,
        request: &CardProfileIds,
    ) -> impl Future<Output = Result<Vec<CardProfile>, GetCardProfileError>> + Send;

    /// gets card profile with a list its linked scryfall data uuid
    fn get_card_profiles_with_scryfall_data_ids(
        &self,
        request: &ScryfallDataIds,
    ) -> impl Future<Output = Result<Vec<CardProfile>, GetCardProfileError>> + Send;

    /// gets last sync date from database
    fn get_last_sync_date(
        &self,
    ) -> impl Future<Output = anyhow::Result<Option<NaiveDateTime>>> + Send;
}

/// orchestrates card related operations
pub trait CardService: Clone + Send + Sync + 'static {
    // ========
    //  create
    // ========

    /// inserts card into database responding with card
    ///
    /// this is not exposed because it is
    /// more for internal unit testing
    fn upsert(
        &self,
        scryfall_data: ScryfallData,
    ) -> impl Future<Output = Result<Card, CreateCardError>>;

    /// syncs database with scryfall bulk data
    fn scryfall_sync(
        &self,
        bulk_endpoint: BulkEndpoint,
    ) -> impl Future<Output = anyhow::Result<SyncMetrics>> + Send;

    // =====
    //  get
    // =====

    /// gets scryfall data with a scryfall data id
    fn get_card(
        &self,
        request: &GetScryfallData,
    ) -> impl Future<Output = Result<Card, GetCardError>> + Send;

    /// gets scryfall cards with a list of card profile ids
    fn get_cards(
        &self,
        request: &ScryfallDataIds,
    ) -> impl Future<Output = Result<Vec<Card>, GetCardError>> + Send;

    /// gets distinct cards matching parameters
    fn search_cards(
        &self,
        request: &CardFilter,
    ) -> impl Future<Output = Result<Vec<Card>, SearchCardsError>> + Send;

    /// gets all distinct artists from cards
    fn get_artists(&self) -> impl Future<Output = Result<Vec<String>, GetArtistsError>> + Send;

    /// gets all distinct types from cards
    fn get_card_types(&self)
        -> impl Future<Output = Result<Vec<String>, GetCardTypesError>> + Send;

    /// gets all distinct sets from cards
    fn get_sets(&self) -> impl Future<Output = Result<Vec<String>, GetSetsError>> + Send;

    /// gets all distinct languages from cards
    fn get_languages(&self) -> impl Future<Output = Result<Vec<String>, GetLanguagesError>> + Send;

    /// gets card profile with its uuid
    fn get_card_profile_with_id(
        &self,
        request: &GetCardProfile,
    ) -> impl Future<Output = Result<CardProfile, GetCardProfileError>> + Send;

    /// gets card profile with a scryfall data id
    fn get_card_profile_with_scryfall_data_id(
        &self,
        request: &GetScryfallData,
    ) -> impl Future<Output = Result<CardProfile, GetCardProfileError>> + Send;

    /// gets card profiles with a list card profile ids
    fn get_card_profiles_with_ids(
        &self,
        request: &CardProfileIds,
    ) -> impl Future<Output = Result<Vec<CardProfile>, GetCardProfileError>> + Send;

    /// gets card profiles with a list of scryfall data ids
    fn get_card_profiles_with_scryfall_data_ids(
        &self,
        request: &ScryfallDataIds,
    ) -> impl Future<Output = Result<Vec<CardProfile>, GetCardProfileError>> + Send;

    /// gets last sync date from database
    fn get_last_sync_date(
        &self,
    ) -> impl Future<Output = anyhow::Result<Option<NaiveDateTime>>> + Send;
}
