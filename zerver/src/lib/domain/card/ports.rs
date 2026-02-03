//! Port traits for MTG card data operations.
//!
//! This module defines the interfaces (ports) for card database and sync operations.
//! All card data originates from the Scryfall API and is synced to the local database
//! for fast querying.
//!
//! # Hexagonal Architecture
//!
//! - **CardRepository**: Database port (card CRUD and Scryfall sync)
//! - **CardService**: Service port (orchestrates sync, search, retrieval)
//!
//! # Card Data Model
//!
//! - **ScryfallData**: Complete MTG card data from Scryfall (~100 fields)
//! - **CardProfile**: User-specific card metadata (favorites, notes - future)
//! - **Card**: Composite (ScryfallData + CardProfile)
//!
//! # Sync Strategy
//!
//! - **Bulk Download**: Fetch all cards from Scryfall bulk endpoint
//! - **Delta Updates**: Only update changed cards (compare timestamps)
//! - **Batch Upsert**: Insert/update in batches to respect PostgreSQL limits
//! - **Metrics Tracking**: Record sync duration, cards processed, errors
//!
//! # Implementation
//!
//! - Repositories: `outbound/sqlx/card` (PostgreSQL + Scryfall API)
//! - Services: `domain/card/services` (sync orchestration + search)

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
///
/// Defines all database operations for card data, including:
/// - Scryfall data sync (bulk download, delta updates, batch insertion)
/// - Card search and retrieval
/// - Distinct value lists (artists, sets, languages, types)
/// - Card profile management
///
/// Implemented by PostgreSQL adapter in `outbound/sqlx/card`.
pub trait CardRepository: Clone + Send + Sync + 'static {
    // ========
    //  create
    // ========

    /// Inserts or updates a single card.
    ///
    /// Creates/updates both ScryfallData and CardProfile records.
    /// Used for individual card operations and testing.
    fn upsert(
        &self,
        scryfall_data: &ScryfallData,
    ) -> impl Future<Output = Result<Card, CreateCardError>> + Send;

    /// Bulk upserts cards without batching.
    ///
    /// **WARNING**: No batching - can hit PostgreSQL parameter limits (~65k params).
    /// Use `batch_upsert` for large datasets. Only use this for small sets (<1000 cards).
    ///
    /// Updates both ScryfallData and CardProfile records.
    fn bulk_upsert(
        &self,
        multiple_scryfall_data: &[ScryfallData],
    ) -> impl Future<Output = Result<Vec<Card>, CreateCardError>> + Send;

    /// Batch upserts cards with transaction batching.
    ///
    /// Processes cards in configurable batch sizes to avoid PostgreSQL limits.
    /// Each batch runs in its own transaction for better error handling.
    ///
    /// Updates sync metrics with progress (cards processed, duration, errors).
    fn batch_upsert(
        &self,
        multiple_scryfall_data: &[ScryfallData],
        batch_size: usize,
        sync_metrics: &mut SyncMetrics,
    ) -> impl Future<Output = Result<Vec<Card>, CreateCardError>> + Send;

    /// Delta upserts - only updates cards newer than database version.
    ///
    /// Compares `edited_at` timestamps and skips cards already up-to-date.
    /// Significantly faster than full sync when most cards unchanged.
    ///
    /// Ideal for incremental syncs after initial bulk load.
    fn batch_delta_upsert(
        &self,
        multiple_scryfall_data: &[ScryfallData],
        batch_size: usize,
        sync_metrics: &mut SyncMetrics,
    ) -> impl Future<Output = Result<Vec<Card>, CreateCardError>> + Send;

    /// Records sync metrics to database for monitoring.
    ///
    /// Stores duration, card counts, errors for each sync operation.
    /// Useful for tracking sync performance over time.
    fn record_sync_metrics(
        &self,
        sync_metrics: &SyncMetrics,
    ) -> impl Future<Output = Result<SyncMetrics, anyhow::Error>> + Send;

    // =====
    //  get
    // =====

    /// Retrieves Scryfall data by Scryfall card ID.
    ///
    /// Returns raw Scryfall card data without CardProfile metadata.
    /// Use `get_card()` for complete Card with profile.
    fn get_scryfall_data(
        &self,
        request: &GetScryfallData,
    ) -> impl Future<Output = Result<ScryfallData, GetScryfallDataError>> + Send;

    /// Retrieves multiple Scryfall data records by IDs.
    ///
    /// Bulk fetch operation for getting raw Scryfall data for multiple cards.
    /// Preserves request order in results.
    fn get_multiple_scryfall_data(
        &self,
        request: &ScryfallDataIds,
    ) -> impl Future<Output = Result<Vec<ScryfallData>, GetScryfallDataError>> + Send;

    /// Searches for Scryfall data matching filter criteria.
    ///
    /// Returns raw Scryfall data without CardProfile metadata.
    /// Supports comprehensive filtering (text, mana, colors, types, etc.).
    fn search_scryfall_data(
        &self,
        request: &CardFilter,
    ) -> impl Future<Output = Result<Vec<ScryfallData>, SearchScryfallDataError>> + Send;

    /// Retrieves complete card (ScryfallData + CardProfile) by Scryfall ID.
    ///
    /// Returns composite Card with both game data and internal metadata.
    fn get_card(
        &self,
        request: &GetScryfallData,
    ) -> impl Future<Output = Result<Card, GetCardError>> + Send;

    /// Retrieves multiple complete cards by Scryfall IDs.
    ///
    /// Bulk fetch operation combining Scryfall data with card profiles.
    /// Preserves request order in results.
    fn get_cards(
        &self,
        request: &ScryfallDataIds,
    ) -> impl Future<Output = Result<Vec<Card>, GetCardError>> + Send;

    /// Searches for complete cards matching filter criteria.
    ///
    /// Returns composite Card objects with both Scryfall data and profiles.
    /// Supports pagination, sorting, and comprehensive filtering.
    fn search_cards(
        &self,
        request: &CardFilter,
    ) -> impl Future<Output = Result<Vec<Card>, SearchCardsError>> + Send;

    /// Retrieves all distinct artist names from card database.
    ///
    /// Used for filter dropdowns and autocomplete. Sorted alphabetically.
    fn get_artists(&self) -> impl Future<Output = Result<Vec<String>, GetArtistsError>> + Send;

    /// Retrieves all distinct card types from card database.
    ///
    /// Returns type strings (e.g., "Creature", "Artifact Creature", "Instant").
    /// Used for filter dropdowns. Sorted alphabetically.
    fn get_card_types(&self)
        -> impl Future<Output = Result<Vec<String>, GetCardTypesError>> + Send;

    /// Retrieves all distinct set codes from card database.
    ///
    /// Returns 3-letter set codes (e.g., "MID", "NEO", "2X2").
    /// Used for filter dropdowns. Sorted alphabetically.
    fn get_sets(&self) -> impl Future<Output = Result<Vec<String>, GetSetsError>> + Send;

    /// Retrieves all distinct language codes from card database.
    ///
    /// Returns 2-letter codes (e.g., "en", "ja", "de").
    /// Used for filter dropdowns. Sorted alphabetically.
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
    ///
    /// Not exposed via HTTP API - primarily for unit tests.
    /// Use `scryfall_sync()` for production card insertion.
    fn upsert(
        &self,
        scryfall_data: ScryfallData,
    ) -> impl Future<Output = Result<Card, CreateCardError>>;

    /// Syncs database with Scryfall bulk data.
    ///
    /// Downloads complete card database from Scryfall and updates local database.
    /// Uses delta sync (only updates changed cards) for efficiency.
    ///
    /// Returns metrics: duration, cards processed, errors encountered.
    ///
    /// # Typical Duration
    ///
    /// - Initial sync: ~5-10 minutes (~100k cards)
    /// - Delta sync: ~30 seconds (only changed cards)
    fn scryfall_sync(
        &self,
        bulk_endpoint: BulkEndpoint,
    ) -> impl Future<Output = anyhow::Result<SyncMetrics>> + Send;

    // =====
    //  get
    // =====

    /// Retrieves complete card (ScryfallData + CardProfile) by Scryfall ID.
    ///
    /// Returns composite Card with both game data and internal metadata.
    fn get_card(
        &self,
        request: &GetScryfallData,
    ) -> impl Future<Output = Result<Card, GetCardError>> + Send;

    /// Retrieves multiple complete cards by Scryfall IDs.
    ///
    /// Bulk fetch operation combining Scryfall data with card profiles.
    /// Preserves request order in results.
    fn get_cards(
        &self,
        request: &ScryfallDataIds,
    ) -> impl Future<Output = Result<Vec<Card>, GetCardError>> + Send;

    /// Searches for complete cards matching filter criteria.
    ///
    /// Returns composite Card objects with both Scryfall data and profiles.
    /// Supports pagination, sorting, and comprehensive filtering.
    fn search_cards(
        &self,
        request: &CardFilter,
    ) -> impl Future<Output = Result<Vec<Card>, SearchCardsError>> + Send;

    /// Retrieves all distinct artist names from card database.
    ///
    /// Used for filter dropdowns and autocomplete. Sorted alphabetically.
    fn get_artists(&self) -> impl Future<Output = Result<Vec<String>, GetArtistsError>> + Send;

    /// Retrieves all distinct card types from card database.
    ///
    /// Returns type strings (e.g., "Creature", "Artifact Creature", "Instant").
    /// Used for filter dropdowns. Sorted alphabetically.
    fn get_card_types(&self)
        -> impl Future<Output = Result<Vec<String>, GetCardTypesError>> + Send;

    /// Retrieves all distinct set codes from card database.
    ///
    /// Returns 3-letter set codes (e.g., "MID", "NEO", "2X2").
    /// Used for filter dropdowns. Sorted alphabetically.
    fn get_sets(&self) -> impl Future<Output = Result<Vec<String>, GetSetsError>> + Send;

    /// Retrieves all distinct language codes from card database.
    ///
    /// Returns 2-letter codes (e.g., "en", "ja", "de").
    /// Used for filter dropdowns. Sorted alphabetically.
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
