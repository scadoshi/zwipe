//! Port traits for MTG card data operations.
//!
//! This module defines the interfaces (ports) for card database and sync operations.
//! All card data originates from the Scryfall API and is synced to the local database
//! for fast querying.

use std::collections::HashMap;
use std::future::Future;

use chrono::{DateTime, Utc};
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::card_profile::CardProfile;
use zwipe_core::domain::card::oracle_tag::OracleTag;
use zwipe_core::domain::card::scryfall_data::ScryfallData;
use zwipe_core::domain::card::search_card::card_filter::CardQuery;

use crate::domain::card::models::search_card::error::SearchCardsError;
use crate::domain::card::models::zervice_metrics::ZerviceMetrics;
use crate::domain::card::requests::create_card::CreateCardError;
use crate::domain::card::requests::get_artists::GetArtistsError;
use crate::domain::card::requests::get_card::GetCardError;
use crate::domain::card::requests::get_card_profile::GetCardProfileError;
use crate::domain::card::requests::get_card_types::GetCardTypesError;
use crate::domain::card::requests::get_keywords::GetKeywordsError;
use crate::domain::card::requests::get_languages::GetLanguagesError;
use crate::domain::card::requests::get_oracle_tags::GetOracleTagsError;
use crate::domain::card::requests::get_oracle_words::GetOracleWordsError;
use crate::domain::card::requests::get_scryfall_data::{
    GetScryfallData, GetScryfallDataError, ScryfallDataIds, SearchScryfallDataError,
};
use crate::domain::card::requests::get_sets::GetSetsError;
use crate::domain::deck::models::deck::commander_suggestions::{
    CommanderSuggestionsError, DeckTagProfile, ScoredCommander,
};
use crate::inbound::external::scryfall::bulk::BulkEndpoint;
use crate::inbound::external::scryfall::oracle_tag::OracleTag as ScryfallOracleTag;

/// The optional deck-serving inputs threaded into the deck-aware search entry
/// points ([`CardRepository::search_scryfall_data_deck_aware`] +
/// [`CardRepository::search_cards_deck_aware`]). Bundled into one struct so those
/// serve paths stay two-arg calls rather than a long positional list. Every field
/// defaults to plain-search behavior (no deck, no scores, no otag term), so a
/// non-deck caller passes [`DeckServeContext::default()`].
#[derive(Debug, Clone, Default)]
pub struct DeckServeContext<'a> {
    /// Deck whose suppression set (skipped / removed cards) is filtered out.
    pub deck_id: Option<uuid::Uuid>,
    /// Oracle ids omitted from the candidate pool.
    pub exclude_oracle_ids: &'a [uuid::Uuid],
    /// Lowercased card name → synergy score. Present ⇒ default synergy ordering.
    pub synergy_scores: Option<&'a serde_json::Value>,
    /// Constrain results to the score map's cards (the synergy membership fence).
    pub synergy_only: bool,
    /// The deck's otag → weight profile (weights sum to 1). Ranks cards
    /// outside the score map by how well their otags cover it; `None` leaves
    /// them ranked by popularity alone.
    pub neighbor_weights: Option<&'a serde_json::Value>,
    /// Commander-select shuffle seed (`{user_id}:{date}`), which enables token/emblem
    /// exclusion + the banded shuffle. `None` on every non-commander-select path.
    pub commander_seed: Option<String>,
    /// The deck's selected oracle tags for the `W_ORACLE_TAG` correlation term.
    pub deck_oracle_tags: &'a [String],
    /// Distinct `card_roles` of the deck's mainboard MVPs, for the `W_STEER`
    /// term. Empty (the default) leaves ordering byte-identical.
    pub mvp_card_roles: &'a [String],
    /// User setting: hide cards with no in-universe printing. The pick
    /// preference in `latest_cards` makes the picked printing's stamp/set a
    /// card-level truth, so the predicate reads the row directly.
    pub exclude_universes_beyond: bool,
    /// Set NAMES whose printings are served despite the exclusion (the
    /// user's whitelisted franchises, expanded via `universe`). Overlapped
    /// against `printing_set_names`.
    pub universes_beyond_exception_set_names: Vec<String>,
}

/// Database port for MTG card operations.
pub trait CardRepository: Clone + Send + Sync + 'static {
    // == create ==

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

    /// Delta upserts: only updates cards newer than database version.
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

    /// Refreshes the `card_signal_rollup` materialized view (pooled per-card
    /// suggestion signal feeding the default synergy ordering).
    fn refresh_card_signal_rollup(&self) -> impl Future<Output = anyhow::Result<()>> + Send;

    /// Refreshes the `otag_context_signal_rollup` materialized view
    /// (generalized-context per-otag signal, the cross-format moat dataset).
    fn refresh_otag_context_signal_rollup(&self)
    -> impl Future<Output = anyhow::Result<()>> + Send;

    /// Full-replaces the `oracle_tags` catalog and the scryfall-sourced rows of
    /// `card_oracle_tags` from a fresh Oracle Tags pull. Returns
    /// `(catalog_rows, correlation_rows)`.
    fn sync_oracle_tags(
        &self,
        tags: &[ScryfallOracleTag],
    ) -> impl Future<Output = anyhow::Result<(usize, usize)>> + Send;

    /// Rebuilds the `card_profiles.oracle_tags` JSONB projection from
    /// `card_oracle_tags` (aggregated per `oracle_id`, fanned to every printing).
    fn refresh_card_oracle_tags(&self) -> impl Future<Output = anyhow::Result<()>> + Send;

    // == get ==

    /// Retrieves Scryfall data by Scryfall card ID.
    fn fetch_scryfall_data(
        &self,
        request: &GetScryfallData,
    ) -> impl Future<Output = Result<ScryfallData, GetScryfallDataError>> + Send;

    /// Retrieves multiple Scryfall data records by IDs.
    fn fetch_multiple_scryfall_data(
        &self,
        request: &ScryfallDataIds,
    ) -> impl Future<Output = Result<Vec<ScryfallData>, GetScryfallDataError>> + Send;

    /// Searches for Scryfall data matching filter criteria.
    fn search_scryfall_data(
        &self,
        request: &CardQuery,
    ) -> impl Future<Output = Result<Vec<ScryfallData>, SearchScryfallDataError>> + Send;

    /// `search_scryfall_data` plus deck-aware extras: `oracle_id` exclusion,
    /// suppression filtering (skipped/removed cards for `deck_id`),
    /// synergy-score default ordering, and (when `synergy_only`) constraining
    /// results to the commander's synergy pool. With `synergy_only` set and a
    /// score map present, the result set is the synergistic cards only, sorted
    /// by the filter's `order_by` (or by synergy score when no sort is given).
    /// The optional deck-serve inputs travel in [`DeckServeContext`]; pass
    /// `DeckServeContext::default()` for a plain search.
    fn search_scryfall_data_deck_aware(
        &self,
        request: &CardQuery,
        context: DeckServeContext<'_>,
    ) -> impl Future<Output = Result<Vec<ScryfallData>, SearchScryfallDataError>> + Send;

    /// Deterministic featured-flavor pick: the id the given hour key selects
    /// from flavor-text cards released in the last 12 months (all-time
    /// fallback so the pool is never empty while any flavor text exists).
    /// Ordering hashes ids with `hour_key`, so the same key always yields the
    /// same card: the serving cache is an optimization, never state.
    fn featured_flavor_id(
        &self,
        hour_key: &str,
    ) -> impl Future<Output = anyhow::Result<Option<uuid::Uuid>>> + Send;

    /// Retrieves complete card by Scryfall ID.
    fn fetch_card(
        &self,
        request: &GetScryfallData,
    ) -> impl Future<Output = Result<Card, GetCardError>> + Send;

    /// Retrieves multiple complete cards by Scryfall IDs.
    fn fetch_cards(
        &self,
        request: &ScryfallDataIds,
    ) -> impl Future<Output = Result<Vec<Card>, GetCardError>> + Send;

    /// Returns all printings of a card by `oracle_id`, ordered by release date.
    fn fetch_printings(
        &self,
        oracle_id: uuid::Uuid,
    ) -> impl Future<Output = Result<Vec<Card>, GetCardError>> + Send;

    /// Retrieves complete cards by oracle ids, at most one card per oracle,
    /// the preferred printing per `latest_cards`. Unknown ids are silently
    /// absent; row order is unspecified.
    fn fetch_latest_cards_by_oracle_ids(
        &self,
        oracle_ids: &[uuid::Uuid],
    ) -> impl Future<Output = Result<Vec<Card>, GetCardError>> + Send;

    /// Searches for complete cards matching filter criteria.
    fn search_cards(
        &self,
        request: &CardQuery,
    ) -> impl Future<Output = Result<Vec<Card>, SearchCardsError>> + Send;

    /// Retrieves all distinct artist names from card database.
    fn fetch_artists(&self) -> impl Future<Output = Result<Vec<String>, GetArtistsError>> + Send;

    /// Retrieves all distinct card types from card database.
    fn fetch_card_types(
        &self,
    ) -> impl Future<Output = Result<Vec<String>, GetCardTypesError>> + Send;

    /// Retrieves all distinct keyword abilities from card database.
    fn fetch_keywords(&self) -> impl Future<Output = Result<Vec<String>, GetKeywordsError>> + Send;

    /// Retrieves the full oracle tag catalog (slug, label, description, parent slugs).
    fn fetch_oracle_tags(
        &self,
    ) -> impl Future<Output = Result<Vec<OracleTag>, GetOracleTagsError>> + Send;

    /// Retrieves all distinct normalized words from oracle text.
    fn fetch_oracle_words(
        &self,
    ) -> impl Future<Output = Result<Vec<String>, GetOracleWordsError>> + Send;

    /// Retrieves all distinct set codes from card database.
    fn fetch_sets(&self) -> impl Future<Output = Result<Vec<String>, GetSetsError>> + Send;

    /// The user's Universes Beyond serving setting: the exclude flag plus
    /// the exception franchises already expanded to set names for the
    /// `printing_set_names` overlap. No settings row = default off.
    fn fetch_universes_beyond_settings(
        &self,
        user_id: uuid::Uuid,
    ) -> impl Future<Output = anyhow::Result<(bool, Vec<String>)>> + Send;

    /// Retrieves all distinct language codes from card database.
    fn fetch_languages(
        &self,
    ) -> impl Future<Output = Result<Vec<String>, GetLanguagesError>> + Send;

    /// Retrieves card profile by Scryfall card UUID.
    ///
    /// Looks up internal metadata using Scryfall card ID as foreign key.
    fn fetch_card_profile(
        &self,
        request: &GetScryfallData,
    ) -> impl Future<Output = Result<CardProfile, GetCardProfileError>> + Send;

    /// Retrieves multiple card profiles by Scryfall card UUIDs.
    ///
    /// Bulk fetch internal metadata using Scryfall IDs as foreign keys.
    fn fetch_card_profiles(
        &self,
        request: &ScryfallDataIds,
    ) -> impl Future<Output = Result<Vec<CardProfile>, GetCardProfileError>> + Send;

    /// Retrieves timestamp of last successful Scryfall sync.
    ///
    /// Returns `None` if no sync has been performed yet.
    /// Used to track sync freshness and schedule next sync.
    fn fetch_last_sync_date(
        &self,
    ) -> impl Future<Output = anyhow::Result<Option<DateTime<Utc>>>> + Send;

    /// Finds cards by exact name match (case-insensitive).
    ///
    /// Returns one card per unique card name, using the latest printing.
    /// Names with no matching card are silently omitted; no `NotFound` error is returned.
    /// Used for bulk import operations where substring match is undesirable.
    fn find_cards_by_exact_names(
        &self,
        names: &[String],
    ) -> impl Future<Output = Result<Vec<Card>, SearchCardsError>> + Send;

    /// The cheapest normal printing of each of `oracle_ids` by its USD price:
    /// English, paper, regular size, not a promo, sold in nonfoil, and never an
    /// Art Series, token or memorabilia printing. A card with no such printing
    /// that has a USD price is left out. Import lands these for a line that
    /// names no printing.
    fn fetch_cheapest_printings(
        &self,
        oracle_ids: &[uuid::Uuid],
    ) -> impl Future<Output = Result<Vec<ScryfallData>, SearchCardsError>> + Send;

    /// Every printing of the cards `oracle_ids` in the sets `set_codes`
    /// (lowercase Scryfall set codes), English first, then paper before
    /// digital, then non-promo, then by collector number. Import resolves a
    /// decklist's `(SET) number` from these.
    fn fetch_printings_in_sets(
        &self,
        oracle_ids: &[uuid::Uuid],
        set_codes: &[String],
    ) -> impl Future<Output = Result<Vec<ScryfallData>, SearchCardsError>> + Send;

    /// `search_cards` with deck awareness: rows whose `oracle_id` is in
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
        context: DeckServeContext<'_>,
    ) -> impl Future<Output = Result<Vec<Card>, SearchCardsError>> + Send;

    /// First-class commander search (`context/archive/commander_select_ordering.md`):
    /// results ordered by decks-helmed popularity, banded + wildcarded per user
    /// per day (deck-independent), with token/emblem printings excluded so a
    /// same-named token is never offered as a commander. An explicit sort in
    /// `request` still wins.
    fn search_commanders(
        &self,
        request: &CardQuery,
        user_id: uuid::Uuid,
    ) -> impl Future<Output = Result<Vec<Card>, SearchCardsError>> + Send;

    /// Fetches the cached synergy payload for a commander by **printing** id
    /// (`scryfall_data.id`, resolved to oracle internally). `None` when the
    /// commander has no cache row yet (graceful absence).
    fn commander_synergy_payload(
        &self,
        commander_printing_id: uuid::Uuid,
    ) -> impl Future<Output = Result<Option<serde_json::Value>, SearchCardsError>> + Send;

    /// Builds a deck's oracle-tag profile from the given cards (by oracle id)
    /// and card names (lowercased): an otag → weight map whose weights sum to
    /// 1, each tag weighted by how often the cards carry it and how rare it is
    /// overall. `None` when the cards carry no usable tags.
    fn deck_tag_profile(
        &self,
        oracle_ids: &[uuid::Uuid],
        lowercased_names: &[String],
    ) -> impl Future<Output = Result<Option<serde_json::Value>, SearchCardsError>> + Send;

    /// Builds one oracle-tag profile per deck in a single query. `decks` holds
    /// each deck's cards by oracle id (duplicates are fine); the result has one
    /// entry per deck in the same order, each an otag → weight map built by
    /// the lift steps in [`zwipe_core::domain::deck::tag_profile`] (tags under
    /// a card role only) with each kept tag's card count, or `None` when they
    /// carry no usable tags. Unlike [`Self::deck_tag_profile`], which ranks
    /// related cards, these profiles are what commander suggestions score.
    fn deck_tag_profiles(
        &self,
        decks: &[Vec<uuid::Uuid>],
    ) -> impl Future<Output = Result<Vec<Option<DeckTagProfile>>, SearchCardsError>> + Send;

    /// Ranks the commanders in `commander_tag_profiles` against a user's
    /// decks, best first, at most `limit`. `decks` is a JSON array of
    /// `{"colors": [WUBRG letters], "profile": {otag: weight} | null}`, one per
    /// deck; a candidate's tag term is its best dot product over the deck
    /// profiles and its color term the best Jaccard match of its color
    /// identity against the deck colors. Leaves out `exclude_oracle_ids`,
    /// tokens, commanders not legal in Commander, and the user's live
    /// dismissals. Each result carries the slugs of its strongest tags shared
    /// with the best-matching deck, each a key of that deck's profile; their
    /// roles come from [`Self::tag_roles`].
    fn score_commander_suggestions(
        &self,
        user_id: uuid::Uuid,
        decks: &serde_json::Value,
        exclude_oracle_ids: &[uuid::Uuid],
        limit: i64,
    ) -> impl Future<Output = Result<Vec<ScoredCommander>, CommanderSuggestionsError>> + Send;

    /// The card role each of `slugs` falls under in the card display's own
    /// membership, the first role alphabetically when under several. Slugs
    /// under no role are absent from the map.
    fn tag_roles(
        &self,
        slugs: &[String],
    ) -> impl Future<Output = Result<HashMap<String, String>, CommanderSuggestionsError>> + Send;

    // == category derivation ==

    /// Fetches a batch of cards by their IDs.
    fn fetch_cards_batch(
        &self,
        ids: &[uuid::Uuid],
    ) -> impl Future<Output = Result<Vec<Card>, anyhow::Error>> + Send;

    /// Updates `card_roles` for a batch of cards.
    fn update_card_roles(
        &self,
        updates: &[(
            uuid::Uuid,
            Vec<zwipe_core::domain::card::card_role::CardRole>,
        )],
    ) -> impl Future<Output = Result<(), anyhow::Error>> + Send;

    /// Rebuilds `card_profiles.card_roles` from Oracle Tags (18 category
    /// subtrees) + `Tokens` via `all_parts`. Returns rows affected. The 4 heuristic
    /// stragglers are merged separately (see `CardService::derive_card_categories`).
    fn derive_oracle_tag_categories(&self) -> impl Future<Output = anyhow::Result<u64>> + Send;

    /// Rebuilds `card_profiles.oracle_tags_by_role` + `.other_oracle_tags` (the
    /// server-grouped card display). Returns rows affected.
    fn refresh_oracle_tag_groups(&self) -> impl Future<Output = anyhow::Result<u64>> + Send;

    /// IDs of every card with oracle text (candidates for the `oracle_tag_gaps` merge).
    fn fetch_card_ids_with_oracle_text(
        &self,
    ) -> impl Future<Output = Result<Vec<uuid::Uuid>, anyhow::Error>> + Send;
}

/// Service port for MTG card business logic.
///
/// Orchestrates card operations including:
/// - **Scryfall Sync**: Downloads and syncs bulk card data from Scryfall API
/// - **Card Search**: Filtering (text, mana, rarity, etc.)
/// - **Card Retrieval**: Get single/multiple cards with full data
/// - **Metadata Lists**: Distinct artists, sets, languages, types
///
/// # Sync Strategy
///
/// `scryfall_sync()` is the primary sync operation:
/// 1. Download bulk JSON from Scryfall (~150MB)
/// 2. Parse JSON into `ScryfallData` structs
/// 3. Batch delta upsert (only update changed cards)
/// 4. Record metrics (duration, cards processed)
///
/// # Implementation
///
/// Implemented in `domain/card/services` with repository calls + Scryfall API client.
pub trait CardService: Clone + Send + Sync + 'static {
    // == create ==

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

    /// Derives `card_roles` from Oracle Tags (18 subtrees + Tokens via
    /// `all_parts`), then merges the 4 heuristic stragglers (Pump/Stax/Protection/
    /// `GraveyardHate`) via `oracle_tag_gaps`. Returns `(otag_rows_written, gap_merges)`.
    fn derive_card_categories(
        &self,
        batch_size: usize,
    ) -> impl Future<Output = anyhow::Result<(u64, usize)>> + Send;

    /// Refreshes the `latest_cards` materialized view after data changes.
    fn refresh_latest_cards(&self) -> impl Future<Output = anyhow::Result<()>> + Send;

    /// Refreshes the `card_signal_rollup` materialized view (nightly, zervice).
    fn refresh_card_signal_rollup(&self) -> impl Future<Output = anyhow::Result<()>> + Send;

    /// Refreshes the `otag_context_signal_rollup` materialized view (nightly, zervice).
    fn refresh_otag_context_signal_rollup(&self)
    -> impl Future<Output = anyhow::Result<()>> + Send;

    /// Syncs the `oracle_tag` catalog + card correlation from Scryfall's Oracle
    /// Tags bulk file (nightly, zervice). Returns `(catalog_rows, correlation_rows)`.
    fn sync_oracle_tags(&self) -> impl Future<Output = anyhow::Result<(usize, usize)>> + Send;

    /// Rebuilds the `card_profiles.oracle_tags` projection (nightly, zervice).
    fn refresh_card_oracle_tags(&self) -> impl Future<Output = anyhow::Result<()>> + Send;

    /// Rebuilds the server-grouped card display (`oracle_tags_by_role` +
    /// `other_oracle_tags`) from the tag hierarchy (nightly, zervice).
    fn refresh_oracle_tag_groups(&self) -> impl Future<Output = anyhow::Result<u64>> + Send;

    // == get ==

    /// The featured-flavor card for an hour bucket; the repository's
    /// deterministic pick assembled like `fetch_card`.
    fn featured_flavor(
        &self,
        hour_key: &str,
    ) -> impl Future<Output = Result<Card, GetCardError>> + Send;

    /// Retrieves complete card by Scryfall ID.
    fn fetch_card(
        &self,
        request: &GetScryfallData,
    ) -> impl Future<Output = Result<Card, GetCardError>> + Send;

    /// Retrieves multiple complete cards by Scryfall IDs.
    fn fetch_cards(
        &self,
        request: &ScryfallDataIds,
    ) -> impl Future<Output = Result<Vec<Card>, GetCardError>> + Send;

    /// Returns all printings of a card by `oracle_id`, ordered by release date.
    fn fetch_printings(
        &self,
        oracle_id: uuid::Uuid,
    ) -> impl Future<Output = Result<Vec<Card>, GetCardError>> + Send;

    /// Searches for complete cards matching filter criteria. Applies the
    /// user's Universes Beyond setting, same as the deck-aware serve and
    /// commander select (the plain search backs the deck form's typed
    /// commander/partner pickers).
    fn search_cards(
        &self,
        request: &CardQuery,
        user_id: uuid::Uuid,
    ) -> impl Future<Output = Result<Vec<Card>, SearchCardsError>> + Send;

    /// One page of an oracle tag's example cards for a reader with no account: the search
    /// [`examples_query`](zwipe_core::domain::card::oracle_tag_examples::examples_query) builds,
    /// with no user's settings.
    fn search_oracle_tag_examples(
        &self,
        request: &CardQuery,
    ) -> impl Future<Output = Result<Vec<Card>, SearchCardsError>> + Send;

    /// Searches for commanders (`context/archive/commander_select_ordering.md`):
    /// popularity-ordered, banded + wildcarded per user per day, token-free.
    fn search_commanders(
        &self,
        request: &CardQuery,
        user_id: uuid::Uuid,
    ) -> impl Future<Output = Result<Vec<Card>, SearchCardsError>> + Send;

    /// Retrieves all distinct artist names from card database.
    fn fetch_artists(&self) -> impl Future<Output = Result<Vec<String>, GetArtistsError>> + Send;

    /// Retrieves all distinct card types from card database.
    fn fetch_card_types(
        &self,
    ) -> impl Future<Output = Result<Vec<String>, GetCardTypesError>> + Send;

    /// Retrieves all distinct keyword abilities from card database.
    fn fetch_keywords(&self) -> impl Future<Output = Result<Vec<String>, GetKeywordsError>> + Send;

    /// Retrieves the full oracle tag catalog (slug, label, description, parent slugs).
    fn fetch_oracle_tags(
        &self,
    ) -> impl Future<Output = Result<Vec<OracleTag>, GetOracleTagsError>> + Send;

    /// Retrieves all distinct normalized words from oracle text.
    fn fetch_oracle_words(
        &self,
    ) -> impl Future<Output = Result<Vec<String>, GetOracleWordsError>> + Send;

    /// Retrieves all distinct set codes from card database.
    fn fetch_sets(&self) -> impl Future<Output = Result<Vec<String>, GetSetsError>> + Send;

    /// Retrieves all distinct language codes from card database.
    fn fetch_languages(
        &self,
    ) -> impl Future<Output = Result<Vec<String>, GetLanguagesError>> + Send;

    /// Retrieves card profile by Scryfall card UUID.
    ///
    /// Looks up internal metadata using Scryfall card ID as foreign key.
    fn fetch_card_profile(
        &self,
        request: &GetScryfallData,
    ) -> impl Future<Output = Result<CardProfile, GetCardProfileError>> + Send;

    /// Retrieves multiple card profiles by Scryfall card UUIDs.
    ///
    /// Bulk fetch internal metadata using Scryfall IDs as foreign keys.
    fn fetch_card_profiles(
        &self,
        request: &ScryfallDataIds,
    ) -> impl Future<Output = Result<Vec<CardProfile>, GetCardProfileError>> + Send;

    /// Retrieves timestamp of last successful Scryfall sync.
    ///
    /// Returns `None` if no sync has been performed yet.
    /// Used to track sync freshness and schedule next sync.
    fn fetch_last_sync_date(
        &self,
    ) -> impl Future<Output = anyhow::Result<Option<DateTime<Utc>>>> + Send;

    /// Finds cards by exact name match (case-insensitive).
    ///
    /// Returns one card per unique card name, using the latest printing.
    /// Names with no matching card are silently omitted; no `NotFound` error is returned.
    /// Used for bulk import operations where substring match is undesirable.
    fn find_cards_by_exact_names(
        &self,
        names: &[String],
    ) -> impl Future<Output = Result<Vec<Card>, SearchCardsError>> + Send;
}
