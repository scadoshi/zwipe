//! Port traits for MTG card data operations.
//!
//! This module defines the interfaces (ports) for card database and sync operations.
//! All card data originates from the Scryfall API and is synced to the local database
//! for fast querying.

use crate::domain::BoxFuture;
use std::future::Future;

use chrono::{DateTime, Utc};

use crate::{
    domain::card::{
        models::{search_card::error::SearchCardsError, zervice_metrics::ZerviceMetrics},
        requests::{
            create_card::CreateCardError,
            get_artists::GetArtistsError,
            get_card::GetCardError,
            get_card_profile::{CardProfileIds, GetCardProfile, GetCardProfileError},
            get_card_types::GetCardTypesError,
            get_keywords::GetKeywordsError,
            get_languages::GetLanguagesError,
            get_oracle_words::GetOracleWordsError,
            get_scryfall_data::{
                GetScryfallData, GetScryfallDataError, ScryfallDataIds, SearchScryfallDataError,
            },
            get_sets::GetSetsError,
        },
    },
    inbound::external::scryfall::bulk::BulkEndpoint,
};
use zwipe_core::domain::card::{
    Card, card_profile::CardProfile, scryfall_data::ScryfallData,
    search_card::card_filter::CardQuery,
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
        zervice_metrics: &mut ZerviceMetrics,
    ) -> impl Future<Output = Result<Vec<Card>, CreateCardError>> + Send;

    /// Delta upserts - only updates cards newer than database version.
    fn batch_delta_upsert(
        &self,
        multiple_scryfall_data: &[ScryfallData],
        batch_size: usize,
        zervice_metrics: &mut ZerviceMetrics,
    ) -> impl Future<Output = Result<Vec<Card>, CreateCardError>> + Send;

    /// Records sync metrics to database.
    fn record_zervice_metrics(
        &self,
        zervice_metrics: &ZerviceMetrics,
    ) -> impl Future<Output = Result<ZerviceMetrics, anyhow::Error>> + Send;

    /// Refreshes the `latest_cards` materialized view.
    ///
    /// Must be called after any operation that changes `scryfall_data` or `card_profiles`
    /// membership (e.g., after Scryfall sync + classification).
    fn refresh_latest_cards(&self) -> impl Future<Output = anyhow::Result<()>> + Send;

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
        request: &CardQuery,
    ) -> impl Future<Output = Result<Vec<ScryfallData>, SearchScryfallDataError>> + Send;

    /// `search_scryfall_data` plus deck-aware extras: oracle_id exclusion,
    /// suppression filtering (skipped/removed cards for `deck_id`),
    /// synergy-score default ordering, and (when `synergy_only`) constraining
    /// results to the commander's synergy pool. With `synergy_only` set and a
    /// score map present, the result set is the synergistic cards only, sorted
    /// by the filter's `order_by` (or by synergy score when no sort is given).
    fn search_scryfall_data_deck_aware(
        &self,
        request: &CardQuery,
        deck_id: Option<uuid::Uuid>,
        exclude_oracle_ids: &[uuid::Uuid],
        synergy_scores: Option<&serde_json::Value>,
        synergy_only: bool,
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

    /// Returns all printings of a card by oracle_id, ordered by release date.
    fn get_printings(
        &self,
        oracle_id: uuid::Uuid,
    ) -> impl Future<Output = Result<Vec<Card>, GetCardError>> + Send;

    /// Searches for complete cards matching filter criteria.
    fn search_cards(
        &self,
        request: &CardQuery,
    ) -> impl Future<Output = Result<Vec<Card>, SearchCardsError>> + Send;

    /// Retrieves all distinct artist names from card database.
    fn get_artists(&self) -> impl Future<Output = Result<Vec<String>, GetArtistsError>> + Send;

    /// Retrieves all distinct card types from card database.
    fn get_card_types(&self)
    -> impl Future<Output = Result<Vec<String>, GetCardTypesError>> + Send;

    /// Retrieves all distinct keyword abilities from card database.
    fn get_keywords(&self) -> impl Future<Output = Result<Vec<String>, GetKeywordsError>> + Send;

    /// Retrieves all distinct normalized words from oracle text.
    fn get_oracle_words(
        &self,
    ) -> impl Future<Output = Result<Vec<String>, GetOracleWordsError>> + Send;

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
    ) -> impl Future<Output = anyhow::Result<Option<DateTime<Utc>>>> + Send;

    /// Finds cards by exact name match (case-insensitive).
    ///
    /// Returns one card per unique card name, using the latest printing.
    /// Names with no matching card are silently omitted — no `NotFound` error is returned.
    /// Used for bulk import operations where substring match is undesirable.
    fn find_cards_by_exact_names(
        &self,
        names: &[String],
    ) -> impl Future<Output = Result<Vec<Card>, SearchCardsError>> + Send;

    /// `search_cards` with deck awareness: rows whose oracle_id is in
    /// `exclude_oracle_ids` are omitted, the deck's suppression set (skipped
    /// / removed cards) is filtered out when `deck_id` is given, and when
    /// `synergy_scores` is given (lowercased card name → score) results are
    /// ordered by score descending with unscored cards last. With
    /// `synergy_only`, results are also constrained to the cards present in
    /// the score map (membership), then sorted by the filter's `order_by`
    /// within that set.
    fn search_cards_deck_aware(
        &self,
        request: &CardQuery,
        deck_id: Option<uuid::Uuid>,
        exclude_oracle_ids: &[uuid::Uuid],
        synergy_scores: Option<&serde_json::Value>,
        synergy_only: bool,
    ) -> impl Future<Output = Result<Vec<Card>, SearchCardsError>> + Send;

    /// Fetches the cached synergy payload for a commander by **printing** id
    /// (`scryfall_data.id` — resolved to oracle internally). `None` when the
    /// commander has no cache row yet (graceful absence).
    fn commander_synergy_payload(
        &self,
        commander_printing_id: uuid::Uuid,
    ) -> impl Future<Output = Result<Option<serde_json::Value>, SearchCardsError>> + Send;

    // ============
    //  classify
    // ============

    /// Retrieves IDs of cards with empty mechanical_categories.
    fn get_unclassified_card_ids(
        &self,
    ) -> impl Future<Output = Result<Vec<uuid::Uuid>, anyhow::Error>> + Send;

    /// Fetches a batch of cards by their IDs.
    fn get_cards_batch(
        &self,
        ids: &[uuid::Uuid],
    ) -> impl Future<Output = Result<Vec<Card>, anyhow::Error>> + Send;

    /// Updates mechanical_categories for a batch of cards.
    fn update_mechanical_categories(
        &self,
        updates: &[(
            uuid::Uuid,
            Vec<zwipe_core::domain::card::mechanical_category::MechanicalCategory>,
        )],
    ) -> impl Future<Output = Result<(), anyhow::Error>> + Send;

    /// Clears all mechanical_categories (resets to empty array).
    fn clear_all_categories(&self) -> impl Future<Output = Result<(), anyhow::Error>> + Send;
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
    ) -> impl Future<Output = Result<Card, CreateCardError>> + Send;

    /// Syncs database with Scryfall bulk data.
    fn scryfall_sync(
        &self,
        bulk_endpoint: BulkEndpoint,
    ) -> impl Future<Output = anyhow::Result<ZerviceMetrics>> + Send;

    /// Classifies cards with empty mechanical_categories using heuristics.
    /// Returns (classified_count, total_untagged_count).
    fn classify_untagged_cards(
        &self,
        batch_size: usize,
    ) -> impl Future<Output = anyhow::Result<(u32, u32)>> + Send;

    /// Clears all mechanical_categories (for --reclassify).
    fn clear_all_categories(&self) -> impl Future<Output = anyhow::Result<()>> + Send;

    /// Refreshes the latest_cards materialized view after data changes.
    fn refresh_latest_cards(&self) -> impl Future<Output = anyhow::Result<()>> + Send;

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

    /// Returns all printings of a card by oracle_id, ordered by release date.
    fn get_printings(
        &self,
        oracle_id: uuid::Uuid,
    ) -> impl Future<Output = Result<Vec<Card>, GetCardError>> + Send;

    /// Searches for complete cards matching filter criteria.
    fn search_cards(
        &self,
        request: &CardQuery,
    ) -> impl Future<Output = Result<Vec<Card>, SearchCardsError>> + Send;

    /// Retrieves all distinct artist names from card database.
    fn get_artists(&self) -> impl Future<Output = Result<Vec<String>, GetArtistsError>> + Send;

    /// Retrieves all distinct card types from card database.
    fn get_card_types(&self)
    -> impl Future<Output = Result<Vec<String>, GetCardTypesError>> + Send;

    /// Retrieves all distinct keyword abilities from card database.
    fn get_keywords(&self) -> impl Future<Output = Result<Vec<String>, GetKeywordsError>> + Send;

    /// Retrieves all distinct normalized words from oracle text.
    fn get_oracle_words(
        &self,
    ) -> impl Future<Output = Result<Vec<String>, GetOracleWordsError>> + Send;

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
    ) -> impl Future<Output = anyhow::Result<Option<DateTime<Utc>>>> + Send;

    /// Finds cards by exact name match (case-insensitive).
    ///
    /// Returns one card per unique card name, using the latest printing.
    /// Names with no matching card are silently omitted — no `NotFound` error is returned.
    /// Used for bulk import operations where substring match is undesirable.
    fn find_cards_by_exact_names(
        &self,
        names: &[String],
    ) -> impl Future<Output = Result<Vec<Card>, SearchCardsError>> + Send;
}

/// Object-safe wrapper used by `AppState` so the concrete service type stays
/// out of the generic parameter list. Auto-implemented for any `CardService`.
pub trait ErasedCardService: Send + Sync + 'static {
    /// See [`CardService::upsert`].
    fn upsert<'a>(
        &'a self,
        scryfall_data: ScryfallData,
    ) -> BoxFuture<'a, Result<Card, CreateCardError>>;

    /// See [`CardService::scryfall_sync`].
    fn scryfall_sync<'a>(
        &'a self,
        bulk_endpoint: BulkEndpoint,
    ) -> BoxFuture<'a, anyhow::Result<ZerviceMetrics>>;

    /// See [`CardService::classify_untagged_cards`].
    fn classify_untagged_cards<'a>(
        &'a self,
        batch_size: usize,
    ) -> BoxFuture<'a, anyhow::Result<(u32, u32)>>;

    /// See [`CardService::clear_all_categories`].
    fn clear_all_categories<'a>(&'a self) -> BoxFuture<'a, anyhow::Result<()>>;

    /// See [`CardService::refresh_latest_cards`].
    fn refresh_latest_cards<'a>(&'a self) -> BoxFuture<'a, anyhow::Result<()>>;

    /// See [`CardService::get_card`].
    fn get_card<'a>(
        &'a self,
        request: &'a GetScryfallData,
    ) -> BoxFuture<'a, Result<Card, GetCardError>>;

    /// See [`CardService::get_cards`].
    fn get_cards<'a>(
        &'a self,
        request: &'a ScryfallDataIds,
    ) -> BoxFuture<'a, Result<Vec<Card>, GetCardError>>;

    /// See [`CardService::get_printings`].
    fn get_printings<'a>(
        &'a self,
        oracle_id: uuid::Uuid,
    ) -> BoxFuture<'a, Result<Vec<Card>, GetCardError>>;

    /// See [`CardService::search_cards`].
    fn search_cards<'a>(
        &'a self,
        request: &'a CardQuery,
    ) -> BoxFuture<'a, Result<Vec<Card>, SearchCardsError>>;

    /// See [`CardService::get_artists`].
    fn get_artists<'a>(&'a self) -> BoxFuture<'a, Result<Vec<String>, GetArtistsError>>;

    /// See [`CardService::get_card_types`].
    fn get_card_types<'a>(&'a self) -> BoxFuture<'a, Result<Vec<String>, GetCardTypesError>>;

    /// See [`CardService::get_keywords`].
    fn get_keywords<'a>(&'a self) -> BoxFuture<'a, Result<Vec<String>, GetKeywordsError>>;

    /// See [`CardService::get_oracle_words`].
    fn get_oracle_words<'a>(&'a self) -> BoxFuture<'a, Result<Vec<String>, GetOracleWordsError>>;

    /// See [`CardService::get_sets`].
    fn get_sets<'a>(&'a self) -> BoxFuture<'a, Result<Vec<String>, GetSetsError>>;

    /// See [`CardService::get_languages`].
    fn get_languages<'a>(&'a self) -> BoxFuture<'a, Result<Vec<String>, GetLanguagesError>>;

    /// See [`CardService::get_card_profile_with_id`].
    fn get_card_profile_with_id<'a>(
        &'a self,
        request: &'a GetCardProfile,
    ) -> BoxFuture<'a, Result<CardProfile, GetCardProfileError>>;

    /// See [`CardService::get_card_profile_with_scryfall_data_id`].
    fn get_card_profile_with_scryfall_data_id<'a>(
        &'a self,
        request: &'a GetScryfallData,
    ) -> BoxFuture<'a, Result<CardProfile, GetCardProfileError>>;

    /// See [`CardService::get_card_profiles_with_ids`].
    fn get_card_profiles_with_ids<'a>(
        &'a self,
        request: &'a CardProfileIds,
    ) -> BoxFuture<'a, Result<Vec<CardProfile>, GetCardProfileError>>;

    /// See [`CardService::get_card_profiles_with_scryfall_data_ids`].
    fn get_card_profiles_with_scryfall_data_ids<'a>(
        &'a self,
        request: &'a ScryfallDataIds,
    ) -> BoxFuture<'a, Result<Vec<CardProfile>, GetCardProfileError>>;

    /// See [`CardService::get_last_sync_date`].
    fn get_last_sync_date<'a>(&'a self) -> BoxFuture<'a, anyhow::Result<Option<DateTime<Utc>>>>;

    /// See [`CardService::find_cards_by_exact_names`].
    fn find_cards_by_exact_names<'a>(
        &'a self,
        names: &'a [String],
    ) -> BoxFuture<'a, Result<Vec<Card>, SearchCardsError>>;
}

impl<T> ErasedCardService for T
where
    T: CardService,
{
    fn upsert<'a>(
        &'a self,
        scryfall_data: ScryfallData,
    ) -> BoxFuture<'a, Result<Card, CreateCardError>> {
        Box::pin(CardService::upsert(self, scryfall_data))
    }

    fn scryfall_sync<'a>(
        &'a self,
        bulk_endpoint: BulkEndpoint,
    ) -> BoxFuture<'a, anyhow::Result<ZerviceMetrics>> {
        Box::pin(CardService::scryfall_sync(self, bulk_endpoint))
    }

    fn classify_untagged_cards<'a>(
        &'a self,
        batch_size: usize,
    ) -> BoxFuture<'a, anyhow::Result<(u32, u32)>> {
        Box::pin(CardService::classify_untagged_cards(self, batch_size))
    }

    fn clear_all_categories<'a>(&'a self) -> BoxFuture<'a, anyhow::Result<()>> {
        Box::pin(CardService::clear_all_categories(self))
    }

    fn refresh_latest_cards<'a>(&'a self) -> BoxFuture<'a, anyhow::Result<()>> {
        Box::pin(CardService::refresh_latest_cards(self))
    }

    fn get_card<'a>(
        &'a self,
        request: &'a GetScryfallData,
    ) -> BoxFuture<'a, Result<Card, GetCardError>> {
        Box::pin(CardService::get_card(self, request))
    }

    fn get_cards<'a>(
        &'a self,
        request: &'a ScryfallDataIds,
    ) -> BoxFuture<'a, Result<Vec<Card>, GetCardError>> {
        Box::pin(CardService::get_cards(self, request))
    }

    fn get_printings<'a>(
        &'a self,
        oracle_id: uuid::Uuid,
    ) -> BoxFuture<'a, Result<Vec<Card>, GetCardError>> {
        Box::pin(CardService::get_printings(self, oracle_id))
    }

    fn search_cards<'a>(
        &'a self,
        request: &'a CardQuery,
    ) -> BoxFuture<'a, Result<Vec<Card>, SearchCardsError>> {
        Box::pin(CardService::search_cards(self, request))
    }

    fn get_artists<'a>(&'a self) -> BoxFuture<'a, Result<Vec<String>, GetArtistsError>> {
        Box::pin(CardService::get_artists(self))
    }

    fn get_card_types<'a>(&'a self) -> BoxFuture<'a, Result<Vec<String>, GetCardTypesError>> {
        Box::pin(CardService::get_card_types(self))
    }

    fn get_keywords<'a>(&'a self) -> BoxFuture<'a, Result<Vec<String>, GetKeywordsError>> {
        Box::pin(CardService::get_keywords(self))
    }

    fn get_oracle_words<'a>(&'a self) -> BoxFuture<'a, Result<Vec<String>, GetOracleWordsError>> {
        Box::pin(CardService::get_oracle_words(self))
    }

    fn get_sets<'a>(&'a self) -> BoxFuture<'a, Result<Vec<String>, GetSetsError>> {
        Box::pin(CardService::get_sets(self))
    }

    fn get_languages<'a>(&'a self) -> BoxFuture<'a, Result<Vec<String>, GetLanguagesError>> {
        Box::pin(CardService::get_languages(self))
    }

    fn get_card_profile_with_id<'a>(
        &'a self,
        request: &'a GetCardProfile,
    ) -> BoxFuture<'a, Result<CardProfile, GetCardProfileError>> {
        Box::pin(CardService::get_card_profile_with_id(self, request))
    }

    fn get_card_profile_with_scryfall_data_id<'a>(
        &'a self,
        request: &'a GetScryfallData,
    ) -> BoxFuture<'a, Result<CardProfile, GetCardProfileError>> {
        Box::pin(CardService::get_card_profile_with_scryfall_data_id(
            self, request,
        ))
    }

    fn get_card_profiles_with_ids<'a>(
        &'a self,
        request: &'a CardProfileIds,
    ) -> BoxFuture<'a, Result<Vec<CardProfile>, GetCardProfileError>> {
        Box::pin(CardService::get_card_profiles_with_ids(self, request))
    }

    fn get_card_profiles_with_scryfall_data_ids<'a>(
        &'a self,
        request: &'a ScryfallDataIds,
    ) -> BoxFuture<'a, Result<Vec<CardProfile>, GetCardProfileError>> {
        Box::pin(CardService::get_card_profiles_with_scryfall_data_ids(
            self, request,
        ))
    }

    fn get_last_sync_date<'a>(&'a self) -> BoxFuture<'a, anyhow::Result<Option<DateTime<Utc>>>> {
        Box::pin(CardService::get_last_sync_date(self))
    }

    fn find_cards_by_exact_names<'a>(
        &'a self,
        names: &'a [String],
    ) -> BoxFuture<'a, Result<Vec<Card>, SearchCardsError>> {
        Box::pin(CardService::find_cards_by_exact_names(self, names))
    }
}
