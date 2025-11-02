use std::future::Future;

use chrono::NaiveDateTime;

use crate::domain::card::models::{
    card_profile::{
        get_card_profile::{CardProfileIds, GetCardProfile, GetCardProfileError},
        CardProfile,
    },
    create_card::CreateCardError,
    get_card::GetCardError,
    get_card_types::GetCardTypesError,
    scryfall_data::{
        get_scryfall_data::{
            GetScryfallData, GetScryfallDataError, ScryfallDataIds, SearchScryfallDataError,
        },
        ScryfallData,
    },
    search_card::{SearchCards, SearchCardsError},
    sync_metrics::{SyncMetrics, SyncType},
    Card,
};

/// enables card related database operations
pub trait CardRepository: Clone + Send + Sync + 'static {
    // ========
    //  create
    // ========

    /// single card creation
    fn insert(
        &self,
        sfd: &ScryfallData,
    ) -> impl Future<Output = Result<Card, CreateCardError>> + Send;

    /// for inserting as many cards as you want :O
    /// - postgres limits parameter counts but that is handled else where
    /// - hence the privateness
    fn bulk_insert(
        &self,
        multiple_scryfall_data: &[ScryfallData],
    ) -> impl Future<Output = Result<Vec<Card>, CreateCardError>> + Send;

    /// for inserting as many cards as you want in batches :O
    /// - chunks into given batch size
    /// - uses bulk_insert internally
    fn batch_insert(
        &self,
        multiple_scryfall_data: &[ScryfallData],
        batch_size: usize,
        sync_metrics: &mut SyncMetrics,
    ) -> impl Future<Output = Result<Vec<Card>, CreateCardError>> + Send;

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
    ) -> impl Future<Output = Result<Vec<Card>, CreateCardError>> + Send;

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
    ) -> impl Future<Output = Result<Vec<Card>, CreateCardError>> + Send;

    /// saves sync_metrics to database
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
        request: &SearchCards,
    ) -> impl Future<Output = Result<Vec<ScryfallData>, SearchScryfallDataError>> + Send;

    /// gets card with a card profile id
    fn get_card(
        &self,
        request: &GetCardProfile,
    ) -> impl Future<Output = Result<Card, GetCardError>> + Send;

    /// gets cards with a list of card profile ids
    fn get_cards(
        &self,
        request: &CardProfileIds,
    ) -> impl Future<Output = Result<Vec<Card>, GetCardError>> + Send;

    /// search for cards given parameters
    fn search_cards(
        &self,
        request: &SearchCards,
    ) -> impl Future<Output = Result<Vec<Card>, SearchCardsError>> + Send;

    /// gets all distinct types from cards
    fn get_card_types(&self)
        -> impl Future<Output = Result<Vec<String>, GetCardTypesError>> + Send;

    /// gets card profile with its uuid
    fn get_card_profile(
        &self,
        request: &GetCardProfile,
    ) -> impl Future<Output = Result<CardProfile, GetCardProfileError>> + Send;

    /// gets card profile with a list of its uuid
    fn get_card_profiles_by_id(
        &self,
        request: &CardProfileIds,
    ) -> impl Future<Output = Result<Vec<CardProfile>, GetCardProfileError>> + Send;

    /// gets card profile with a list its linked scryfall data uuid
    fn get_card_profiles_by_scryfall_data_id(
        &self,
        request: &ScryfallDataIds,
    ) -> impl Future<Output = Result<Vec<CardProfile>, GetCardProfileError>> + Send;

    /// gets last sync date from database
    fn get_last_sync_date(
        &self,
        sync_type: SyncType,
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
    fn insert(
        &self,
        scryfall_data: ScryfallData,
    ) -> impl Future<Output = Result<Card, CreateCardError>>;

    /// syncs database with scryfall bulk data
    fn scryfall_sync(
        &self,
        sync_type: SyncType,
    ) -> impl Future<Output = anyhow::Result<SyncMetrics>> + Send;

    // =====
    //  get
    // =====

    /// gets scryfall data with a card profile id
    fn get_card(
        &self,
        request: &GetCardProfile,
    ) -> impl Future<Output = Result<Card, GetCardError>> + Send;

    /// gets scryfall cards with a list of card profile ids
    fn get_cards(
        &self,
        request: &CardProfileIds,
    ) -> impl Future<Output = Result<Vec<Card>, GetCardError>> + Send;

    /// gets cards matching parameters
    fn search_cards(
        &self,
        request: &SearchCards,
    ) -> impl Future<Output = Result<Vec<Card>, SearchCardsError>> + Send;

    /// gets all distinct types from cards
    fn get_card_types(&self)
        -> impl Future<Output = Result<Vec<String>, GetCardTypesError>> + Send;

    /// gets card profile with a uuid
    fn get_card_profile(
        &self,
        request: &GetCardProfile,
    ) -> impl Future<Output = Result<CardProfile, GetCardProfileError>> + Send;

    /// gets card profiles with a list of uuids
    fn get_card_profiles(
        &self,
        request: &CardProfileIds,
    ) -> impl Future<Output = Result<Vec<CardProfile>, GetCardProfileError>> + Send;

    /// gets last sync date from database
    fn get_last_sync_date(
        &self,
        sync_type: SyncType,
    ) -> impl Future<Output = anyhow::Result<Option<NaiveDateTime>>> + Send;
}
