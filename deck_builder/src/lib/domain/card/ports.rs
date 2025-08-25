use std::future::Future;

use chrono::NaiveDateTime;
use uuid::Uuid;

use crate::domain::card::models::{
    scryfall_card::ScryfallCard,
    sync_metrics::{SyncMetrics, SyncType},
    CardSearchParameters, CreateCardError, GetCardError, SearchCardError,
};

pub trait CardRepository: Clone + Send + Sync + 'static {
    /// for testing :)
    fn insert_with_card_response(
        &self,
        card: ScryfallCard,
    ) -> impl Future<Output = Result<ScryfallCard, CreateCardError>>;

    /// simple single card insert with no return value
    fn insert(
        &self,
        card: ScryfallCard,
    ) -> impl Future<Output = Result<(), CreateCardError>> + Send;

    /// for inserting multiple cards
    /// beware as you can overload the database insertion query
    /// if you try too many cards at once
    /// that is what batch_insert is for
    fn bulk_insert(
        &self,
        cards: Vec<ScryfallCard>,
    ) -> impl Future<Output = Result<(), CreateCardError>> + Send;

    /// for inserting multiple cards
    /// batches them in smaller groups for fewer cards per query inserting
    /// utilizes bulk_insert internally
    fn batch_insert(
        &self,
        cards: Vec<ScryfallCard>,
        batch_size: usize,
        sync_metrics: &mut SyncMetrics,
    ) -> impl Future<Output = Result<(), CreateCardError>> + Send;

    /// reads inbound cards
    /// reads database cards
    /// removes cards from inbound which already exist in database
    /// intended to insert only new cards
    fn batch_insert_if_not_exists(
        &self,
        cards: Vec<ScryfallCard>,
        batch_size: usize,
        sync_metrics: &mut SyncMetrics,
    ) -> impl Future<Output = Result<(), CreateCardError>> + Send;

    /// reads inbound cards
    /// deletes database's versions of those cards
    /// inserts all inbound cards
    /// intended to allow for a refresh of the database
    /// with card data that is possibly more up to date
    fn delete_if_exists_and_batch_insert(
        &self,
        cards: Vec<ScryfallCard>,
        batch_size: usize,
        sync_metrics: &mut SyncMetrics,
    ) -> impl Future<Output = Result<(), CreateCardError>> + Send;

    /// simple card get by id
    fn get_card(
        &self,
        id: &Uuid,
    ) -> impl Future<Output = Result<ScryfallCard, GetCardError>> + Send;

    /// simple card search by a list of parameters
    fn search_cards(
        &self,
        params: CardSearchParameters,
    ) -> impl Future<Output = Result<Vec<ScryfallCard>, SearchCardError>> + Send;

    /// delete all cards
    fn delete_all(&self) -> impl Future<Output = Result<(), anyhow::Error>> + Send;

    fn record_sync_metrics(
        &self,
        sync_metrics: SyncMetrics,
    ) -> impl Future<Output = Result<(), anyhow::Error>> + Send;

    fn get_last_sync_date(
        &self,
        sync_type: SyncType,
    ) -> impl Future<Output = anyhow::Result<Option<NaiveDateTime>>> + Send;
}

pub trait CardService {
    /// for testing :)
    fn insert_with_card_response(
        &self,
        card: ScryfallCard,
    ) -> impl Future<Output = Result<ScryfallCard, CreateCardError>>;

    fn get_card(
        &self,
        id: &Uuid,
    ) -> impl Future<Output = Result<ScryfallCard, GetCardError>> + Send;

    fn search_cards(
        &self,
        params: CardSearchParameters,
    ) -> impl Future<Output = Result<Vec<ScryfallCard>, SearchCardError>> + Send;

    fn scryfall_sync(&self, sync_type: SyncType)
        -> impl Future<Output = anyhow::Result<()>> + Send;

    fn get_last_sync_date(
        &self,
        sync_type: SyncType,
    ) -> impl Future<Output = anyhow::Result<Option<NaiveDateTime>>> + Send;
}
