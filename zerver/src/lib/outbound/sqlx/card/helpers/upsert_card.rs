//! Card upsert helpers for PostgreSQL transactions.
//!
//! Provides multiple strategies for inserting/updating Scryfall card data:
//! - **Single**: One card at a time with full error reporting
//! - **Bulk**: Multiple cards in a single query for maximum throughput
//! - **Batch**: Chunked processing with automatic fallback on failure
//! - **Delta**: Only upserts cards that have actually changed
//!
//! All operations work within existing transactions (caller commits).

use crate::{
    domain::card::{
        models::{
            helpers::SleeveScryfallData,
            zervice_metrics::{ErrorMetrics, ZerviceMetrics},
        },
        requests::{create_card::CreateCardError, get_scryfall_data::ScryfallDataIds},
    },
    outbound::sqlx::card::{
        card_profile::DatabaseCardProfile,
        helpers::scryfall_data_fields::{
            BindCards, BindScryfallDataFields, bulk_upsert_conflict_fields, scryfall_data_fields,
        },
        models::DatabaseScryfallData,
    },
};
use sqlx::{PgTransaction, QueryBuilder, query_as};
use std::future::Future;
use zwipe_core::domain::card::{Card, card_profile::CardProfile, scryfall_data::ScryfallData};

/// Postgres error substring used to filter noise from card-by-card fallback retries.
///
/// When a transaction is aborted, subsequent statements produce this error instead
/// of the root cause — so these are silently skipped during error reporting.
const POSTGRES_TX_ABORT_MESSAGE: &str = "current transaction is aborted";

// ====================
//  validation helpers
// ====================

/// Determines if a card is a token based on Scryfall layout field.
fn is_token(scryfall_data: &ScryfallData) -> bool {
    scryfall_data.layout == "token"
}

// ===========
//  insertion
// ===========

// below allows redundant operations within `CardRepository`
// without having to create new transactions
// these should **not** commit the transaction

/// Single-card insert/update within an existing transaction.
///
/// Upserts one [`ScryfallData`] record and its associated [`CardProfile`],
/// returning the combined [`Card`]. Use this for one-off inserts or as a
/// fallback when bulk operations fail.
pub trait SingleUpsertWithTx
where
    Self: Sized,
{
    /// Inserts or updates this card within the given transaction.
    ///
    /// Also creates/updates the card profile with computed `is_token` flag.
    fn single_upsert_with_tx(
        &self,
        tx: &mut PgTransaction<'_>,
    ) -> impl Future<Output = Result<Card, CreateCardError>> + Send;
}

impl SingleUpsertWithTx for ScryfallData {
    async fn single_upsert_with_tx(
        &self,
        tx: &mut PgTransaction<'_>,
    ) -> Result<Card, CreateCardError> {
        let scryfall_data_id = self.id;
        let mut sfd_qb = QueryBuilder::new("INSERT INTO scryfall_data (");
        sfd_qb.push(scryfall_data_fields()).push(") VALUES ");
        sfd_qb.bind_scryfall_fields(self);
        sfd_qb.push(bulk_upsert_conflict_fields());
        sfd_qb.push(" RETURNING *");
        let db: DatabaseScryfallData = sfd_qb
            .build_query_as::<DatabaseScryfallData>()
            .fetch_one(&mut **tx)
            .await?;
        let scryfall_data: ScryfallData =
            db.try_into().map_err(CreateCardError::ScryfallDataFromDb)?;
        let is_token = is_token(&scryfall_data);
        let database_card_profile = query_as!(
            DatabaseCardProfile,
            "INSERT INTO card_profiles (scryfall_data_id, is_token)
             VALUES ($1, $2)
             ON CONFLICT (scryfall_data_id)
             DO UPDATE SET updated_at = NOW(), is_token = EXCLUDED.is_token
             RETURNING scryfall_data_id, is_token, card_roles, oracle_tags, oracle_tags_by_role, other_oracle_tags, created_at, updated_at",
            scryfall_data_id,
            is_token
        )
        .fetch_one(&mut **tx)
        .await?;
        let card_profile: CardProfile = database_card_profile.into();
        let card = Card::new(card_profile, scryfall_data);
        Ok(card)
    }
}

/// Bulk insert/update of multiple cards in a single query.
///
/// Deduplicates input by `id` before insertion to avoid constraint conflicts
/// within the same batch. Most efficient for large imports when all cards
/// are expected to succeed.
pub trait BulkUpsertWithTx
where
    Self: Sized,
{
    /// Upserts all cards in a single SQL statement.
    ///
    /// Returns the list of upserted cards. Fails atomically if any card
    /// causes a constraint violation.
    fn bulk_upsert_with_tx(
        &self,
        tx: &mut PgTransaction<'_>,
    ) -> impl Future<Output = Result<Vec<Card>, CreateCardError>> + Send;
}

impl BulkUpsertWithTx for &[ScryfallData] {
    async fn bulk_upsert_with_tx(
        &self,
        tx: &mut PgTransaction<'_>,
    ) -> Result<Vec<Card>, CreateCardError> {
        let mut scryfall_data: Vec<ScryfallData> = self.to_vec();
        scryfall_data.dedup_by_key(|scryfall| scryfall.id);
        let mut scryfall_data_query_builder = QueryBuilder::new("INSERT INTO scryfall_data (");
        scryfall_data_query_builder
            .push(scryfall_data_fields())
            .push(") VALUES ")
            .bind_cards(scryfall_data.as_slice())
            .push(bulk_upsert_conflict_fields())
            .push(" RETURNING *;");
        let db_rows: Vec<DatabaseScryfallData> = scryfall_data_query_builder
            .build_query_as::<DatabaseScryfallData>()
            .fetch_all(&mut **tx)
            .await?;
        let database_scryfall_data: Vec<ScryfallData> = db_rows
            .into_iter()
            .map(ScryfallData::try_from)
            .collect::<Result<_, _>>()
            .map_err(CreateCardError::ScryfallDataFromDb)?;
        let mut card_profile_query_builder =
            QueryBuilder::new("INSERT INTO card_profiles (scryfall_data_id, is_token) VALUES");
        for (i, scryfall_data) in database_scryfall_data.iter().enumerate() {
            if i > 0 {
                card_profile_query_builder.push(",");
            }
            let is_token = is_token(scryfall_data);
            card_profile_query_builder
                .push("(")
                .push_bind(scryfall_data.id)
                .push(",")
                .push_bind(is_token)
                .push(")");
        }
        card_profile_query_builder
            .push(" ON CONFLICT (scryfall_data_id) DO UPDATE SET updated_at = NOW(), is_token = EXCLUDED.is_token ");
        card_profile_query_builder.push(
            " RETURNING scryfall_data_id, is_token, card_roles, oracle_tags, created_at, updated_at;",
        );
        let card_profiles: Vec<CardProfile> = card_profile_query_builder
            .build_query_as::<DatabaseCardProfile>()
            .fetch_all(&mut **tx)
            .await?
            .into_iter()
            .map(|database_card_profile| database_card_profile.into())
            .collect();
        let cards: Vec<Card> = scryfall_data.sleeve(card_profiles);
        Ok(cards)
    }
}

/// Chunked batch insert/update with automatic fallback.
///
/// Processes cards in chunks using [`BulkUpsertWithTx`]. On chunk failure,
/// falls back to card-by-card insertion via [`upsert_card_by_card`] so one
/// bad card doesn't block the entire batch.
pub trait BatchUpsertWithTx
where
    Self: Sized,
{
    /// Upserts cards in batches, tracking metrics and handling failures gracefully.
    ///
    /// Updates `zervice_metrics` with counts of upserted cards and any errors encountered.
    fn batch_upsert_with_tx(
        self,
        tx: &mut PgTransaction<'_>,
        batch_size: usize,
        zervice_metrics: &mut ZerviceMetrics,
    ) -> impl Future<Output = Result<Vec<Card>, CreateCardError>> + Send;
}

/// Fallback function that inserts cards one at a time.
///
/// Used when bulk operations fail. Logs individual errors while continuing
/// to process remaining cards. Silently skips transaction-aborted errors
/// since they're not the root cause.
async fn upsert_card_by_card(
    batch: &[ScryfallData],
    tx: &mut PgTransaction<'_>,
    zervice_metrics: &mut ZerviceMetrics,
) {
    for card in batch {
        match card.single_upsert_with_tx(tx).await {
            Ok(_) => {
                zervice_metrics.add_upserted_count(1);
            }
            Err(e) => {
                // ignore tx abort messages as they are never root cause
                if !e.to_string().contains(POSTGRES_TX_ABORT_MESSAGE) {
                    let error = ErrorMetrics::new(card.id, &card.name, e.to_string());
                    tracing::warn!("insertion failure => {}", error);
                    zervice_metrics.add_error(error);
                }
            }
        }
    }
}

impl BatchUpsertWithTx for &[ScryfallData] {
    async fn batch_upsert_with_tx(
        self,
        tx: &mut PgTransaction<'_>,
        batch_size: usize,
        zervice_metrics: &mut ZerviceMetrics,
    ) -> Result<Vec<Card>, CreateCardError> {
        let mut cards: Vec<Card> = Vec::new();
        for chunk in self.chunks(batch_size) {
            match chunk.bulk_upsert_with_tx(tx).await {
                Ok(upserted) => {
                    let upserted_count = upserted.len();
                    cards.extend(upserted);
                    zervice_metrics.add_upserted_count(upserted_count as i32);
                }
                Err(e) => {
                    tracing::warn!("batch failed with error: {:?}\nretrying card by card", e);
                    upsert_card_by_card(chunk, tx, zervice_metrics).await;
                }
            }
        }
        Ok(cards)
    }
}

/// Delta-aware bulk upsert that skips unchanged cards.
///
/// Fetches existing records by ID, compares with input, and only upserts the
/// diff. Returns both the upserted cards and a count of skipped (unchanged) cards.
/// Ideal for incremental sync operations.
pub trait BulkDeltaUpsertWithTx
where
    Self: Sized,
{
    /// Upserts only cards that differ from their existing database records.
    ///
    /// Returns a tuple of (upserted cards, skipped count).
    fn bulk_delta_upsert_with_tx(
        self,
        tx: &mut PgTransaction<'_>,
    ) -> impl Future<Output = Result<(Vec<Card>, usize), CreateCardError>> + Send;
}

impl BulkDeltaUpsertWithTx for &[ScryfallData] {
    async fn bulk_delta_upsert_with_tx(
        self,
        tx: &mut PgTransaction<'_>,
    ) -> Result<(Vec<Card>, usize), CreateCardError> {
        let existing_db: Vec<DatabaseScryfallData> =
            query_as("SELECT * FROM scryfall_data WHERE id = ANY($1)")
                .bind(&*ScryfallDataIds::from(self))
                .fetch_all(&mut **tx)
                .await
                .map_err(|e| CreateCardError::GetScryfallData(e.into()))?;
        let existing: Vec<ScryfallData> = existing_db
            .into_iter()
            .map(ScryfallData::try_from)
            .collect::<Result<_, _>>()
            .map_err(CreateCardError::ScryfallDataFromDb)?;
        let delta: Vec<ScryfallData> = self
            .iter()
            .filter(|x| !existing.contains(x))
            .map(|x| x.to_owned())
            .collect();
        let skipped_count = self.len() - delta.len();
        if delta.is_empty() {
            return Ok((Vec::new(), skipped_count));
        }
        let cards = delta.as_slice().bulk_upsert_with_tx(tx).await?;
        Ok((cards, skipped_count))
    }
}

/// Combines chunked batching with delta detection.
///
/// Each chunk runs through [`BulkDeltaUpsertWithTx`] (skip unchanged, upsert diff).
/// On chunk failure, falls back to card-by-card insertion. Best for large
/// incremental syncs where most cards haven't changed.
pub trait BatchDeltaUpsertWithTx
where
    Self: Sized,
{
    /// Batch-processes cards with delta detection and automatic fallback.
    ///
    /// Updates `zervice_metrics` with upserted, skipped, and error counts.
    fn batch_delta_upsert_with_tx(
        self,
        tx: &mut PgTransaction<'_>,
        batch_size: usize,
        zervice_metrics: &mut ZerviceMetrics,
    ) -> impl Future<Output = Result<Vec<Card>, CreateCardError>> + Send;
}

impl BatchDeltaUpsertWithTx for &[ScryfallData] {
    async fn batch_delta_upsert_with_tx(
        self,
        tx: &mut PgTransaction<'_>,
        batch_size: usize,
        zervice_metrics: &mut ZerviceMetrics,
    ) -> Result<Vec<Card>, CreateCardError> {
        let mut cards: Vec<Card> = Vec::new();
        for chunk in self.chunks(batch_size) {
            match chunk.bulk_delta_upsert_with_tx(tx).await {
                Ok((upserted, skipped)) => {
                    zervice_metrics.add_upserted_count(upserted.len() as i32);
                    zervice_metrics.add_skipped_count(skipped as i32);
                    cards.extend(upserted);
                }
                Err(e) => {
                    tracing::warn!("batch failed with error: {:?}\nretrying card by card", e);
                    upsert_card_by_card(chunk, tx, zervice_metrics).await;
                }
            }
        }
        Ok(cards)
    }
}
