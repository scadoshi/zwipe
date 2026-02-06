//! Port traits for MTG card data operations.
//!
//! This module defines the interfaces (ports) for card database and sync operations.
//! All card data originates from the Scryfall API and is synced to the local database
//! for fast querying.

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

/// Database port for MTG card operations.
pub trait CardRepository: Clone + Send + Sync + 'static {
    // ========
    //  create
    // ========

    /// Inserts or updates a single card.
    fn upsert(
        &self,
        scryfall_data: &ScryfallData,
    ) -> impl Future<Output = Result<Card, CreateCardError>> + Send;

    /// Bulk upserts cards without batching.
    fn bulk_upsert(
        &self,
        multiple_scryfall_data: &[ScryfallData],
    ) -> impl Future<Output = Result<Vec<Card>, CreateCardError>> + Send;

    /// Batch upserts cards with transaction batching.
    fn batch_upsert(
        &self,
        multiple_scryfall_data: &[ScryfallData],
        batch_size: usize,
        sync_metrics: &mut SyncMetrics,
    ) -> impl Future<Output = Result<Vec<Card>, CreateCardError>> + Send;

    /// Delta upserts - only updates cards newer than database version.
    fn batch_delta_upsert(
        &self,
        multiple_scryfall_data: &[ScryfallData],
        batch_size: usize,
        sync_metrics: &mut SyncMetrics,
    ) -> impl Future<Output = Result<Vec<Card>, CreateCardError>> + Send;

    /// Records sync metrics to database.
    fn record_sync_metrics(
        &self,
        sync_metrics: &SyncMetrics,
    ) -> impl Future<Output = Result<SyncMetrics, anyhow::Error>> + Send;

    // =====
    //  get
    // =====

    /// Retrieves Scryfall data by Scryfall card ID.
    fn get_scryfall_data(
        &self,
        request: &GetScryfallData,
    ) -> impl Future<Output = Result<ScryfallData, GetScryfallDataError>> + Send;

    /// Retrieves multiple Scryfall data records by IDs.
    fn get_multiple_scryfall_data(
        &self,
        request: &ScryfallDataIds,
    ) -> impl Future<Output = Result<Vec<ScryfallData>, GetScryfallDataError>> + Send;

    /// Searches for Scryfall data matching filter criteria.
    fn search_scryfall_data(
        &self,
        request: &CardFilter,
    ) -> impl Future<Output = Result<Vec<ScryfallData>, SearchScryfallDataError>> + Send;

    /// Retrieves complete card by Scryfall ID.
    fn get_card(
        &self,
        request: &GetScryfallData,
    ) -> impl Future<Output = Result<Card, GetCardError>> + Send;

    /// Retrieves multiple complete cards by Scryfall IDs.
    fn get_cards(
        &self,
        request: &ScryfallDataIds,
    ) -> impl Future<Output = Result<Vec<Card>, GetCardError>> + Send;

    /// Searches for complete cards matching filter criteria.
    fn search_cards(
        &self,
        request: &CardFilter,
    ) -> impl Future<Output = Result<Vec<Card>, SearchCardsError>> + Send;

    /// Retrieves all distinct artist names from card database.
    fn get_artists(&self) -> impl Future<Output = Result<Vec<String>, GetArtistsError>> + Send;

    /// Retrieves all distinct card types from card database.
    fn get_card_types(&self)
        -> impl Future<Output = Result<Vec<String>, GetCardTypesError>> + Send;

    /// Retrieves all distinct set codes from card database.
    fn get_sets(&self) -> impl Future<Output = Result<Vec<String>, GetSetsError>> + Send;

    /// Retrieves all distinct language codes from card database.
    fn get_languages(&self) -> impl Future<Output = Result<Vec<String>, GetLanguagesError>> + Send;

    /// Retrieves card profile by card profile UUID (internal ID).
    ///
    /// Returns internal metadata (sync timestamps, DB ID) without Scryfall data.
    fn get_card_profile_with_id(
        &self,
        request: &GetCardProfile,
    ) -> impl Future<Output = Result<CardProfile, GetCardProfileError>> + Send;

    /// Retrieves card profile by Scryfall card UUID.
    ///
    /// Looks up internal metadata using Scryfall card ID as foreign key.
    fn get_card_profile_with_scryfall_data_id(
        &self,
        request: &GetScryfallData,
    ) -> impl Future<Output = Result<CardProfile, GetCardProfileError>> + Send;

    /// Retrieves multiple card profiles by card profile UUIDs.
    ///
    /// Bulk fetch operation for internal metadata.
    fn get_card_profiles_with_ids(
        &self,
        request: &CardProfileIds,
    ) -> impl Future<Output = Result<Vec<CardProfile>, GetCardProfileError>> + Send;

    /// Retrieves multiple card profiles by Scryfall card UUIDs.
    ///
    /// Bulk fetch internal metadata using Scryfall IDs as foreign keys.
    fn get_card_profiles_with_scryfall_data_ids(
        &self,
        request: &ScryfallDataIds,
    ) -> impl Future<Output = Result<Vec<CardProfile>, GetCardProfileError>> + Send;

    /// Retrieves timestamp of last successful Scryfall sync.
    ///
    /// Returns `None` if no sync has been performed yet.
    /// Used to track sync freshness and schedule next sync.
    fn get_last_sync_date(
        &self,
    ) -> impl Future<Output = anyhow::Result<Option<NaiveDateTime>>> + Send;
}

/// Service port for MTG card business logic.
///
/// Orchestrates card operations including:
/// - **Scryfall Sync**: Downloads and syncs bulk card data from Scryfall API
/// - **Card Search**: Comprehensive filtering (text, mana, rarity, etc.)
/// - **Card Retrieval**: Get single/multiple cards with full data
/// - **Metadata Lists**: Distinct artists, sets, languages, types
///
/// # Sync Strategy
///
/// `scryfall_sync()` is the primary sync operation:
/// 1. Download bulk JSON from Scryfall (~150MB)
/// 2. Parse JSON into ScryfallData structs
/// 3. Batch delta upsert (only update changed cards)
/// 4. Record metrics (duration, cards processed)
///
/// # Implementation
///
/// Implemented in `domain/card/services` with repository calls + Scryfall API client.
pub trait CardService: Clone + Send + Sync + 'static {
    // ========
    //  create
    // ========

    /// Inserts a single card (internal/testing only).
    fn upsert(
        &self,
        scryfall_data: ScryfallData,
    ) -> impl Future<Output = Result<Card, CreateCardError>>;

    /// Syncs database with Scryfall bulk data.
    fn scryfall_sync(
        &self,
        bulk_endpoint: BulkEndpoint,
    ) -> impl Future<Output = anyhow::Result<SyncMetrics>> + Send;

    // =====
    //  get
    // =====

    /// Retrieves complete card by Scryfall ID.
    fn get_card(
        &self,
        request: &GetScryfallData,
    ) -> impl Future<Output = Result<Card, GetCardError>> + Send;

    /// Retrieves multiple complete cards by Scryfall IDs.
    fn get_cards(
        &self,
        request: &ScryfallDataIds,
    ) -> impl Future<Output = Result<Vec<Card>, GetCardError>> + Send;

    /// Searches for complete cards matching filter criteria.
    fn search_cards(
        &self,
        request: &CardFilter,
    ) -> impl Future<Output = Result<Vec<Card>, SearchCardsError>> + Send;

    /// Retrieves all distinct artist names from card database.
    fn get_artists(&self) -> impl Future<Output = Result<Vec<String>, GetArtistsError>> + Send;

    /// Retrieves all distinct card types from card database.
    fn get_card_types(&self)
        -> impl Future<Output = Result<Vec<String>, GetCardTypesError>> + Send;

    /// Retrieves all distinct set codes from card database.
    fn get_sets(&self) -> impl Future<Output = Result<Vec<String>, GetSetsError>> + Send;

    /// Retrieves all distinct language codes from card database.
    fn get_languages(&self) -> impl Future<Output = Result<Vec<String>, GetLanguagesError>> + Send;

    /// Retrieves card profile by card profile UUID (internal ID).
    ///
    /// Returns internal metadata (sync timestamps, DB ID) without Scryfall data.
    fn get_card_profile_with_id(
        &self,
        request: &GetCardProfile,
    ) -> impl Future<Output = Result<CardProfile, GetCardProfileError>> + Send;

    /// Retrieves card profile by Scryfall card UUID.
    ///
    /// Looks up internal metadata using Scryfall card ID as foreign key.
    fn get_card_profile_with_scryfall_data_id(
        &self,
        request: &GetScryfallData,
    ) -> impl Future<Output = Result<CardProfile, GetCardProfileError>> + Send;

    /// Retrieves multiple card profiles by card profile UUIDs.
    ///
    /// Bulk fetch operation for internal metadata.
    fn get_card_profiles_with_ids(
        &self,
        request: &CardProfileIds,
    ) -> impl Future<Output = Result<Vec<CardProfile>, GetCardProfileError>> + Send;

    /// Retrieves multiple card profiles by Scryfall card UUIDs.
    ///
    /// Bulk fetch internal metadata using Scryfall IDs as foreign keys.
    fn get_card_profiles_with_scryfall_data_ids(
        &self,
        request: &ScryfallDataIds,
    ) -> impl Future<Output = Result<Vec<CardProfile>, GetCardProfileError>> + Send;

    /// Retrieves timestamp of last successful Scryfall sync.
    ///
    /// Returns `None` if no sync has been performed yet.
    /// Used to track sync freshness and schedule next sync.
    fn get_last_sync_date(
        &self,
    ) -> impl Future<Output = anyhow::Result<Option<NaiveDateTime>>> + Send;
}
