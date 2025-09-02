use std::future::Future;

use chrono::NaiveDateTime;

use crate::domain::card::models::{
    card_profile::{
        CardProfile, GetCardProfileError, GetCardProfileRequest, GetCardProfilesRequest,
    },
    scryfall_data::{
        CreateCardError, CreateScryfallDataError, GetMultipleScryfallDataRequest,
        GetScryfallDataError, GetScryfallDataRequest, ScryfallData, SearchCardError,
        SearchCardRequest, SearchScryfallDataError, SearchScryfallDataRequest,
    },
    sync_metrics::{SyncMetrics, SyncType},
    Card,
};

/// enables card related database operations
pub trait CardRepository: Clone + Send + Sync + 'static {
    /// simple card insert
    fn insert(
        &self,
        scryfall_data: &ScryfallData,
    ) -> impl Future<Output = Result<ScryfallData, CreateScryfallDataError>> + Send;

    /// for inserting as many cards as you want (*3*)
    /// - postgres limits parameter counts but that is handled else where
    /// - hence the privateness
    fn bulk_insert(
        &self,
        multiple_scryfall_data: &[ScryfallData],
    ) -> impl Future<Output = Result<Vec<ScryfallData>, CreateScryfallDataError>> + Send;

    /// for inserting as many cards as you want in batches (*3*)
    /// - chunks into given batch size
    /// - uses bulk_insert internally
    fn batch_insert(
        &self,
        multiple_scryfall_data: &[ScryfallData],
        batch_size: usize,
        sync_metrics: &mut SyncMetrics,
    ) -> impl Future<Output = Result<Vec<ScryfallData>, CreateScryfallDataError>> + Send;

    /// intends to incrementally update database with only new cards
    ///
    /// the flow looks something like this
    /// 1. find ids of cards which
    /// given list include but database does not
    /// 2. insert **only those** cards into database
    /// (uses batch_insert)
    ///
    /// transactions keep it atomic!
    fn batch_insert_if_not_exists(
        &self,
        multiple_scryfall_data: &[ScryfallData],
        batch_size: usize,
        sync_metrics: &mut SyncMetrics,
    ) -> impl Future<Output = Result<Vec<ScryfallData>, CreateScryfallDataError>> + Send;

    /// intends to refresh cards in database with the given list of cards
    ///
    /// the flow looks something like this
    /// 1. find ids of cards which given list *and* database share
    /// 2. delete cards with those ids from database
    /// 3. insert all given cards back into database
    /// (uses batch_insert)
    ///
    /// transactions keep it atomic!
    fn delete_if_exists_and_batch_insert(
        &self,
        multiple_scryfall_data: &[ScryfallData],
        batch_size: usize,
        sync_metrics: &mut SyncMetrics,
    ) -> impl Future<Output = Result<Vec<ScryfallData>, CreateScryfallDataError>> + Send;

    /// gets scryfall card with a uuid
    fn get_scryfall_data(
        &self,
        request: &GetScryfallDataRequest,
    ) -> impl Future<Output = Result<ScryfallData, GetScryfallDataError>> + Send;

    /// gets scryfall cards with a list of uuids
    fn get_multiple_scryfall_data(
        &self,
        request: &GetMultipleScryfallDataRequest,
    ) -> impl Future<Output = Result<Vec<ScryfallData>, GetScryfallDataError>> + Send;

    /// search for cards given parameters
    fn search_cards(
        &self,
        request: &SearchCardRequest,
    ) -> impl Future<Output = Result<Vec<Card>, SearchCardError>> + Send;

    /// gets card profile with a uuid
    fn get_card_profile(
        &self,
        request: &GetCardProfileRequest,
    ) -> impl Future<Output = Result<CardProfile, GetCardProfileError>> + Send;

    /// gets card profiles with a list of uuids
    fn get_card_profiles(
        &self,
        request: &GetCardProfilesRequest,
    ) -> impl Future<Output = Result<Vec<CardProfile>, GetCardProfileError>> + Send;

    /// saves sync_metrics to database
    fn record_sync_metrics(
        &self,
        sync_metrics: SyncMetrics,
    ) -> impl Future<Output = Result<SyncMetrics, anyhow::Error>> + Send;

    /// gets last sync date from database
    fn get_last_sync_date(
        &self,
        sync_type: SyncType,
    ) -> impl Future<Output = anyhow::Result<Option<NaiveDateTime>>> + Send;
}

/// orchestrates card related operations
pub trait CardService: Clone + Send + Sync + 'static {
    /// inserts card into database responding with card
    ///
    /// this is not exposed because it is
    /// more for internal unit testing
    fn insert(
        &self,
        scryfall_data: ScryfallData,
    ) -> impl Future<Output = Result<Card, CreateCardError>>;

    /// gets scryfall data with a uuid
    fn get_scryfall_data(
        &self,
        request: &GetScryfallDataRequest,
    ) -> impl Future<Output = Result<ScryfallData, GetScryfallDataError>> + Send;

    /// gets scryfall cards with a list of uuids
    fn get_multiple_scryfall_data(
        &self,
        request: &GetMultipleScryfallDataRequest,
    ) -> impl Future<Output = Result<Vec<ScryfallData>, GetScryfallDataError>> + Send;

    /// gets cards matching parameters
    fn search_cards(
        &self,
        request: &SearchCardRequest,
    ) -> impl Future<Output = Result<Vec<Card>, SearchCardError>> + Send;

    /// gets card profile with a uuid
    fn get_card_profile(
        &self,
        request: &GetCardProfileRequest,
    ) -> impl Future<Output = Result<CardProfile, GetCardProfileError>> + Send;

    /// gets card profiles with a list of uuids
    fn get_card_profiles(
        &self,
        request: &GetCardProfilesRequest,
    ) -> impl Future<Output = Result<Vec<CardProfile>, GetCardProfileError>> + Send;

    /// syncs database with scryfall bulk data
    fn scryfall_sync(
        &self,
        sync_type: SyncType,
    ) -> impl Future<Output = anyhow::Result<SyncMetrics>> + Send;

    /// gets last sync date from database
    fn get_last_sync_date(
        &self,
        sync_type: SyncType,
    ) -> impl Future<Output = anyhow::Result<Option<NaiveDateTime>>> + Send;
}
