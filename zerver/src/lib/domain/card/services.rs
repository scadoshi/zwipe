use std::collections::HashSet;

use anyhow::bail;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::card_profile::CardProfile;
use zwipe_core::domain::card::card_role::{self, CardRole};
use zwipe_core::domain::card::models::lookup::{CardTypeWord, LOOKUP_LIMIT, LOOKUP_PAGE};
use zwipe_core::domain::card::oracle_tag::OracleTag;
use zwipe_core::domain::card::scryfall_data::ScryfallData;
use zwipe_core::domain::card::search_card::card_filter::CardQuery;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::card_sort_key::CardSortKey;
use zwipe_core::domain::card::search_card::card_filter::error::InvalidCardCriteria;
use zwipe_core::domain::card::slug::CardSlug;
use zwipe_core::http::contracts::card::HttpCardLookup;

use crate::domain::card::models::printing_removal::{self, RemovalMode, RemovalReport};
use crate::domain::card::models::scryfall_pull::{self, CatalogCounts, ScryfallLoad};
use crate::domain::card::models::search_card::error::{LookupCardsError, SearchCardsError};
use crate::domain::card::models::zervice_metrics::{self, SyncedCards, ZerviceMetrics};
use crate::domain::card::models::zervice_step::StepRecord;
use crate::domain::card::ports::{CardRepository, CardService};
use crate::domain::card::requests::create_card::CreateCardError;
use crate::domain::card::requests::get_artists::GetArtistsError;
use crate::domain::card::requests::get_card::GetCardError;
use crate::domain::card::requests::get_card_profile::GetCardProfileError;
use crate::domain::card::requests::get_card_types::GetCardTypesError;
use crate::domain::card::requests::get_keywords::GetKeywordsError;
use crate::domain::card::requests::get_languages::GetLanguagesError;
use crate::domain::card::requests::get_oracle_tags::GetOracleTagsError;
use crate::domain::card::requests::get_oracle_words::GetOracleWordsError;
use crate::domain::card::requests::get_scryfall_data::{GetScryfallData, ScryfallDataIds};
use crate::domain::card::requests::get_sets::GetSetsError;
use crate::inbound::external::scryfall::bulk::{self, BulkEndpoint, BulkSource};
use crate::inbound::external::scryfall::oracle_tag::OracleTag as ScryfallOracleTag;
use crate::outbound::sqlx::card::helpers::scryfall_data_fields;

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
/// With 87 fields per card and the 65,535 param limit, this yields 376 cards per batch.
fn batch_size() -> usize {
    // At least one card per batch, even with no fields to bind.
    (POSTGRESQL_PARAMETER_HARD_LIMIT / 2)
        .checked_div(scryfall_data_fields::scryfall_data_field_count())
        .map_or(1, |size| size.max(1))
}

/// Sort key for a role: its position in the enum's declaration.
#[expect(
    clippy::as_conversions,
    reason = "a fieldless enum's discriminant; its 27 variants fit in u8"
)]
fn role_order(role: CardRole) -> u8 {
    role as u8
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
/// Batch size is auto-calculated based on `ScryfallData` field count (376 cards/batch).
#[derive(Debug, Clone)]
pub struct Service<R> {
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
    // == create ==
    async fn upsert(&self, scryfall_data: ScryfallData) -> Result<Card, CreateCardError> {
        self.repo.upsert(&scryfall_data).await
    }

    async fn scryfall_sync<S: BulkSource>(&self, source: &S) -> anyhow::Result<SyncedCards> {
        let file = source.describe(BulkEndpoint::DefaultCards).await?;
        tracing::info!("performing scryfall sync with {}", file.kind);
        if let Some(made) = file.updated_at {
            let last = self.repo.fetch_scryfall_load(file.kind).await?;
            scryfall_pull::check_fresh(file.kind, made, last)?;
        }
        let mut zervice_metrics = ZerviceMetrics::new();
        let batch_size = batch_size();
        let pulled = bulk::pull_cards(source, file).await?;
        let mut scryfall_data: Vec<ScryfallData> = pulled.items;
        zervice_metrics.set_received_count(zervice_metrics::count_column(scryfall_data.len()));
        let stored = self.repo.fetch_catalog_counts().await?;
        scryfall_pull::check_cards(&scryfall_data, stored)?;
        let kept_prices =
            scryfall_pull::prices_collapsed(CatalogCounts::of(&scryfall_data), stored);
        if let Some(collapse) = &kept_prices {
            tracing::error!("{collapse}");
            let ids: Vec<Uuid> = scryfall_data.iter().map(|sd| sd.id).collect();
            let mut stored_prices = self.repo.fetch_stored_prices(&ids).await?;
            for card in &mut scryfall_data {
                if let Some(prices) = stored_prices.remove(&card.id) {
                    card.prices = prices;
                }
            }
        }
        let load = pulled.file.updated_at.map(|made| ScryfallLoad {
            kind: pulled.file.kind,
            updated_at: made,
            uri: pulled.file.uri.clone(),
            items: u64::try_from(scryfall_data.len()).unwrap_or(u64::MAX),
        });
        self.repo
            .batch_delta_upsert(
                &scryfall_data,
                batch_size,
                &mut zervice_metrics,
                load.as_ref(),
            )
            .await?;
        zervice_metrics.mark_as_completed();
        let zervice_metrics = self.repo.record_zervice_metrics(&zervice_metrics).await?;
        tracing::info!("sync complete: {}", zervice_metrics);
        Ok(SyncedCards {
            metrics: zervice_metrics,
            printing_ids: scryfall_data.iter().map(|sd| sd.id).collect(),
            kept_prices,
        })
    }

    async fn record_zervice_step(&self, record: &StepRecord) -> anyhow::Result<()> {
        self.repo.record_zervice_step(record).await
    }

    async fn remove_deleted_printings(
        &self,
        bulk: &HashSet<Uuid>,
        mode: RemovalMode,
    ) -> anyhow::Result<RemovalReport> {
        if bulk.is_empty() {
            bail!("the bulk file listed no printings; removing nothing");
        }
        let stored = self.repo.fetch_stored_printings().await?;
        let gone = printing_removal::gone(&stored, bulk);
        let removed: Vec<Uuid> = gone.iter().map(|p| p.id).collect();
        let oracles: Vec<Uuid> = gone
            .iter()
            .filter_map(|p| p.oracle_id)
            .collect::<HashSet<Uuid>>()
            .into_iter()
            .collect();
        let survivors = self
            .repo
            .fetch_printing_survivors(&oracles, &removed)
            .await?;
        let references = self.repo.fetch_printing_references(&removed).await?;
        let plan = printing_removal::plan(gone, &survivors, &references);
        let refused = scryfall_pull::check_removal(
            u64::try_from(plan.removals.len()).unwrap_or(u64::MAX),
            u64::try_from(stored.len()).unwrap_or(u64::MAX),
        )
        .err();
        let counts = match (mode, &refused) {
            (RemovalMode::Apply, Some(refusal)) => return Err(refusal.clone().into()),
            (RemovalMode::Apply, None) => Some(self.repo.apply_printing_removals(&plan).await?),
            (RemovalMode::Preview, _) => None,
        };
        Ok(RemovalReport {
            plan,
            counts,
            refused,
        })
    }

    async fn derive_card_categories(&self, batch_size: usize) -> anyhow::Result<(u64, usize)> {
        // 1. Oracle-tag subtrees (18) + Tokens via all_parts: one SQL pass, all cards.
        let otag_rows = self.repo.derive_oracle_tag_categories().await?;

        // 2. Merge the 4 heuristic stragglers otags can't express (oracle_tag_gaps).
        let ids = self.repo.fetch_card_ids_with_oracle_text().await?;
        let mut merged = 0usize;
        for chunk in ids.chunks(batch_size) {
            let cards = self.repo.fetch_cards_batch(chunk).await?;
            let updates: Vec<_> = cards
                .iter()
                .filter_map(|card| {
                    let gaps = card_role::classify_oracle_tag_gaps(card);
                    if gaps.is_empty() {
                        return None;
                    }
                    let mut union: Vec<CardRole> = card
                        .card_profile
                        .card_roles
                        .iter()
                        .filter_map(|s| CardRole::try_from(s.as_str()).ok())
                        .collect();
                    let before = union.len();
                    for g in gaps {
                        if !union.contains(&g) {
                            union.push(g);
                        }
                    }
                    if union.len() == before {
                        return None; // gaps already present, nothing to write
                    }
                    union.sort_by_key(|role| role_order(*role));
                    Some((card.scryfall_data.id, union))
                })
                .collect();
            merged = merged.saturating_add(updates.len());
            if !updates.is_empty() {
                self.repo.update_card_roles(&updates).await?;
            }
        }
        Ok((otag_rows, merged))
    }

    async fn refresh_latest_cards(&self) -> anyhow::Result<()> {
        self.repo.refresh_latest_cards().await
    }

    async fn refresh_card_signal_rollup(&self) -> anyhow::Result<()> {
        self.repo.refresh_card_signal_rollup().await
    }

    async fn refresh_otag_context_signal_rollup(&self) -> anyhow::Result<()> {
        self.repo.refresh_otag_context_signal_rollup().await
    }

    async fn sync_oracle_tags<S: BulkSource>(&self, source: &S) -> anyhow::Result<(usize, usize)> {
        tracing::info!("performing oracle tags sync");
        let file = source.describe(BulkEndpoint::OracleTags).await?;
        if let Some(made) = file.updated_at {
            let last = self.repo.fetch_scryfall_load(file.kind).await?;
            scryfall_pull::check_fresh(file.kind, made, last)?;
        }
        let pulled = bulk::pull::<ScryfallOracleTag, _>(source, file).await?;
        tracing::info!("fetched {} oracle tags", pulled.items.len());
        let load = pulled.file.updated_at.map(|made| ScryfallLoad {
            kind: pulled.file.kind,
            updated_at: made,
            uri: pulled.file.uri.clone(),
            items: u64::try_from(pulled.items.len()).unwrap_or(u64::MAX),
        });
        self.repo
            .sync_oracle_tags(&pulled.items, load.as_ref())
            .await
    }

    async fn sync_oracle_tags_from<S: BulkSource>(
        &self,
        source: &S,
        url: &str,
    ) -> anyhow::Result<(usize, usize)> {
        tracing::info!("performing oracle tags sync from {url}");
        let tags: Vec<ScryfallOracleTag> = bulk::pull(source, bulk::oracle_tags_file(url)?)
            .await?
            .items;
        tracing::info!("fetched {} oracle tags", tags.len());
        // An explicit older file: no age check, and no load recorded, so the mark never lowers.
        self.repo.sync_oracle_tags(&tags, None).await
    }

    async fn refresh_card_oracle_tags(&self) -> anyhow::Result<u64> {
        self.repo.refresh_card_oracle_tags().await
    }

    async fn refresh_oracle_tag_groups(&self) -> anyhow::Result<u64> {
        self.repo.refresh_oracle_tag_groups().await
    }

    // == get ==
    async fn featured_flavor(&self, hour_key: &str) -> Result<Card, GetCardError> {
        use crate::domain::card::requests::get_scryfall_data::GetScryfallDataError;
        let id = self
            .repo
            .featured_flavor_id(hour_key)
            .await
            .map_err(GetScryfallDataError::Database)?
            .ok_or(GetScryfallDataError::NotFound)?;
        let request = GetScryfallData::from(id);
        self.repo.fetch_card(&request).await
    }

    async fn fetch_card(&self, request: &GetScryfallData) -> Result<Card, GetCardError> {
        self.repo.fetch_card(request).await
    }

    async fn fetch_cards(&self, request: &ScryfallDataIds) -> Result<Vec<Card>, GetCardError> {
        self.repo.fetch_cards(request).await
    }

    async fn fetch_printings(&self, oracle_id: uuid::Uuid) -> Result<Vec<Card>, GetCardError> {
        self.repo.fetch_printings(oracle_id).await
    }

    async fn fetch_named_cards(&self, slug: &CardSlug) -> Result<Vec<Card>, GetCardError> {
        self.repo.fetch_named_cards(slug).await
    }

    async fn search_cards(
        &self,
        request: &CardQuery,
        user_id: uuid::Uuid,
    ) -> Result<Vec<Card>, SearchCardsError> {
        // The Universes Beyond setting applies here too: this path backs
        // the deck form's typed commander/partner pickers, and typed-name
        // intent does not override a "never serve crossover cards" setting
        // (unlike suppressions, which typed Quick add deliberately bypasses).
        // A failed read degrades to serving everything.
        let (exclude_universes_beyond, universes_beyond_exception_set_names) = self
            .repo
            .fetch_universes_beyond_settings(user_id)
            .await
            .unwrap_or_default();
        self.repo
            .search_cards_deck_aware(
                request,
                crate::domain::card::ports::DeckServeContext {
                    exclude_universes_beyond,
                    universes_beyond_exception_set_names,
                    ..Default::default()
                },
            )
            .await
    }

    async fn search_oracle_tag_examples(
        &self,
        request: &CardQuery,
    ) -> Result<Vec<Card>, SearchCardsError> {
        self.repo.search_cards(request).await
    }

    async fn search_commanders(
        &self,
        request: &CardQuery,
        user_id: uuid::Uuid,
    ) -> Result<Vec<Card>, SearchCardsError> {
        self.repo.search_commanders(request, user_id).await
    }

    async fn fetch_artists(&self) -> Result<Vec<String>, GetArtistsError> {
        self.repo.fetch_artists().await
    }

    async fn fetch_card_types(&self) -> Result<Vec<String>, GetCardTypesError> {
        self.repo.fetch_card_types().await
    }

    async fn fetch_keywords(&self) -> Result<Vec<String>, GetKeywordsError> {
        self.repo.fetch_keywords().await
    }

    async fn fetch_oracle_tags(&self) -> Result<Vec<OracleTag>, GetOracleTagsError> {
        self.repo.fetch_oracle_tags().await
    }

    async fn fetch_oracle_words(&self) -> Result<Vec<String>, GetOracleWordsError> {
        self.repo.fetch_oracle_words().await
    }

    async fn fetch_sets(&self) -> Result<Vec<String>, GetSetsError> {
        self.repo.fetch_sets().await
    }

    async fn fetch_languages(&self) -> Result<Vec<String>, GetLanguagesError> {
        self.repo.fetch_languages().await
    }

    async fn fetch_card_profile(
        &self,
        request: &GetScryfallData,
    ) -> Result<CardProfile, GetCardProfileError> {
        self.repo.fetch_card_profile(request).await
    }

    async fn fetch_card_profiles(
        &self,
        request: &ScryfallDataIds,
    ) -> Result<Vec<CardProfile>, GetCardProfileError> {
        self.repo.fetch_card_profiles(request).await
    }

    async fn fetch_last_sync_date(&self) -> anyhow::Result<Option<DateTime<Utc>>> {
        self.repo.fetch_last_sync_date().await
    }

    async fn find_cards_by_exact_names(
        &self,
        names: &[String],
    ) -> Result<Vec<Card>, SearchCardsError> {
        self.repo.find_cards_by_exact_names(names).await
    }

    async fn lookup_cards(
        &self,
        lookup: &HttpCardLookup,
        user_id: Option<Uuid>,
    ) -> Result<Vec<Card>, LookupCardsError> {
        let query = lookup_query(lookup)?;
        // Signed out, nothing is excluded; a failed settings read degrades to the same.
        let (exclude_universes_beyond, universes_beyond_exception_set_names) = match user_id {
            Some(user_id) => self
                .repo
                .fetch_universes_beyond_settings(user_id)
                .await
                .unwrap_or_default(),
            None => Default::default(),
        };
        Ok(self
            .repo
            .search_cards_deck_aware(
                &query,
                crate::domain::card::ports::DeckServeContext {
                    exclude_universes_beyond,
                    universes_beyond_exception_set_names,
                    ..Default::default()
                },
            )
            .await?)
    }
}

/// The query a lookup runs, its chips read as the filter's are by default: a type line holding
/// any of the types, a color identity equal to the colors, and any of the roles. With no offset,
/// the first [`LOOKUP_LIMIT`] cards; with one, [`LOOKUP_PAGE`] cards from it. Best match first: the whole name, then a name whose first word is
/// the name typed, then one that starts with it, then one with a word that does, then the rest,
/// each tier by popularity.
fn lookup_query(lookup: &HttpCardLookup) -> Result<CardQuery, InvalidCardCriteria> {
    let mut builder = CardQueryBuilder::new();
    builder
        .set_name_contains(lookup.name().as_str())
        .set_type_line_contains_any(lookup.types().iter().map(CardTypeWord::as_str))
        .set_card_roles_contains_any(lookup.card_roles().iter().map(ToString::to_string))
        .set_sort(CardSortKey::BestMatch);
    match lookup.offset() {
        Some(offset) => builder.set_limit(LOOKUP_PAGE).set_offset(offset),
        None => builder.set_limit(LOOKUP_LIMIT),
    };
    // No colors leaves identity open; an empty set would mean colorless.
    if !lookup.colors().is_empty() {
        builder.set_color_identity_equals(lookup.colors().iter().copied().collect());
    }
    builder.build()
}

#[cfg(test)]
mod lookup_tests {
    use zwipe_core::domain::card::models::lookup::LookupName;

    use super::*;

    fn bolt() -> HttpCardLookup {
        let name = LookupName::try_from("bolt".to_owned()).unwrap();
        HttpCardLookup::try_new(name, Vec::new(), Vec::new(), Vec::new()).unwrap()
    }

    #[test]
    fn a_lookup_with_no_offset_answers_the_first_thirty_as_before() {
        let query = lookup_query(&bolt()).unwrap();
        assert_eq!(query.limit(), LOOKUP_LIMIT);
        assert_eq!(query.offset(), 0);
    }

    #[test]
    fn a_paged_lookup_answers_a_page_of_twenty_from_its_offset() {
        let query = lookup_query(&bolt().paged(40)).unwrap();
        assert_eq!(query.limit(), LOOKUP_PAGE);
        assert_eq!(query.offset(), 40);
        let first = lookup_query(&bolt().paged(0)).unwrap();
        assert_eq!(
            first.limit(),
            LOOKUP_PAGE,
            "a pager's first page is a page too"
        );
        assert_eq!(first.offset(), 0);
    }
}
