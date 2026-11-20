//! Card upsert helpers for PostgreSQL transactions.
//!
//! Provides multiple strategies for inserting/updating Scryfall card data:
//! - **Single**: One card at a time with full error reporting
//! - **Bulk**: Multiple cards in a single query for maximum throughput
//! - **Batch**: Chunked processing with automatic fallback on failure
//! - **Delta**: Only upserts cards that have actually changed
//!
//! All operations work within existing transactions (caller commits).

use std::future::Future;

use sqlx::postgres::PgDatabaseError;
use sqlx::{Acquire, PgTransaction, QueryBuilder, query_as};
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::card_profile::CardProfile;
use zwipe_core::domain::card::scryfall_data::ScryfallData;

use crate::domain::card::models::helpers::SleeveScryfallData;
use crate::domain::card::models::zervice_metrics::{self, ErrorMetrics, ZerviceMetrics};
use crate::domain::card::requests::create_card::CreateCardError;
use crate::domain::card::requests::get_scryfall_data::ScryfallDataIds;
use crate::outbound::sqlx::card::card_profile::DatabaseCardProfile;
use crate::outbound::sqlx::card::helpers::scryfall_data_fields::{
    self, BindCards, BindScryfallDataFields,
};
use crate::outbound::sqlx::card::models::DatabaseScryfallData;

// == validation helpers ==

/// Determines if a card is a token based on Scryfall layout field.
fn is_token(scryfall_data: &ScryfallData) -> bool {
    scryfall_data.layout == "token"
}

// == insertion ==

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
        sfd_qb
            .push(scryfall_data_fields::scryfall_data_fields())
            .push(") VALUES ");
        sfd_qb.bind_scryfall_fields(self);
        sfd_qb.push(scryfall_data_fields::bulk_upsert_conflict_fields());
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
            .push(scryfall_data_fields::scryfall_data_fields())
            .push(") VALUES ")
            .bind_cards(scryfall_data.as_slice())
            .push(scryfall_data_fields::bulk_upsert_conflict_fields())
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
            " RETURNING scryfall_data_id, is_token, card_roles, oracle_tags, oracle_tags_by_role, other_oracle_tags, created_at, updated_at;",
        );
        let card_profiles: Vec<CardProfile> = card_profile_query_builder
            .build_query_as::<DatabaseCardProfile>()
            .fetch_all(&mut **tx)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();
        let cards: Vec<Card> = scryfall_data.sleeve(card_profiles);
        Ok(cards)
    }
}

/// Chunked batch insert/update with automatic fallback.
///
/// Processes cards in chunks using [`BulkUpsertWithTx`]. On chunk failure,
/// falls back to card-by-card insertion via `upsert_card_by_card` so one
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
        match upsert_one_in_savepoint(card, tx).await {
            Ok(()) => {
                zervice_metrics.add_upserted_count(1);
            }
            Err(e) => {
                let error = ErrorMetrics::new(card.id, &card.name, e.to_string());
                tracing::warn!("insertion failure => {}", error);
                zervice_metrics.add_error(error);
            }
        }
    }
}

/// Upserts `card` inside its own savepoint, so a failure rolls back only the card and leaves
/// `tx` usable for the next one.
async fn upsert_one_in_savepoint(
    card: &ScryfallData,
    tx: &mut PgTransaction<'_>,
) -> Result<(), CreateCardError> {
    let mut savepoint = tx.begin().await?;
    match card.single_upsert_with_tx(&mut savepoint).await {
        Ok(_) => {
            savepoint.commit().await?;
            Ok(())
        }
        Err(e) => {
            savepoint.rollback().await?;
            Err(label_clash(e, tx).await)
        }
    }
}

/// The `scryfall_data` columns a clash may name and be looked up by, each with its type, so the
/// lookup compares in that type and its index serves it: each column Postgres holds unique, plus
/// the store ids a clash could plausibly name.
const CLASH_COLUMNS: &[(&str, &str)] = &[
    ("id", "uuid"),
    ("mtgo_id", "integer"),
    ("mtgo_foil_id", "integer"),
    ("arena_id", "integer"),
    ("tcgplayer_id", "integer"),
];

/// The printing whose `column` holds `value`, as its id and name. The lookup runs in its own
/// savepoint, so a failure, which reads as no holder, never aborts the sync's transaction.
async fn fetch_holder(
    tx: &mut PgTransaction<'_>,
    column: &str,
    kind: &str,
    value: &str,
) -> Option<(Uuid, String)> {
    let mut savepoint = tx.begin().await.ok()?;
    // The column and its type come from CLASH_COLUMNS, never from the error text.
    let sql =
        format!("SELECT id, name FROM scryfall_data WHERE {column} = CAST($1 AS {kind}) LIMIT 1");
    let holder = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(value)
        .fetch_optional(&mut *savepoint)
        .await;
    if let Ok(found) = holder {
        savepoint.commit().await.ok()?;
        found
    } else {
        savepoint.rollback().await.ok()?;
        None
    }
}

/// The column and value in a unique violation's detail, `Key (mtgo_id)=(777) already exists.`
fn clash_of(detail: &str) -> Option<(&str, &str)> {
    let rest = detail.strip_prefix("Key (")?;
    let (column, rest) = rest.split_once(")=(")?;
    let value = rest.strip_suffix(") already exists.")?;
    Some((column, value))
}

/// Turns a unique violation into a [`CreateCardError::Clash`] naming the column, its value and
/// the printing holding it. Any other error, or a detail it can't read, passes through.
async fn label_clash(error: CreateCardError, tx: &mut PgTransaction<'_>) -> CreateCardError {
    let CreateCardError::UniqueConstraintViolation(cause) = &error else {
        return error;
    };
    let detail = cause
        .downcast_ref::<sqlx::Error>()
        .and_then(sqlx::Error::as_database_error)
        .and_then(|db| db.try_downcast_ref::<PgDatabaseError>())
        .and_then(PgDatabaseError::detail)
        .map(ToOwned::to_owned);
    let Some((column, value)) = detail.as_deref().and_then(clash_of) else {
        return error;
    };
    let Some(&(column, kind)) = CLASH_COLUMNS.iter().find(|(c, _)| *c == column) else {
        return error;
    };
    let holder = fetch_holder(tx, column, kind, value).await;
    CreateCardError::Clash {
        column: column.to_string(),
        value: value.to_string(),
        holder: holder.map_or_else(
            || "another printing".to_string(),
            |(id, name)| format!("{name} ({id})"),
        ),
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
            // The batch runs in a savepoint: a failure rolls back to it, so the card-by-card
            // retry runs on a usable transaction.
            let mut savepoint = tx.begin().await?;
            let attempt = chunk.bulk_upsert_with_tx(&mut savepoint).await;
            match &attempt {
                Ok(_) => savepoint.commit().await?,
                Err(_) => savepoint.rollback().await?,
            }
            match attempt {
                Ok(upserted) => {
                    let upserted_count = upserted.len();
                    cards.extend(upserted);
                    zervice_metrics
                        .add_upserted_count(zervice_metrics::count_column(upserted_count));
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
            .map(ToOwned::to_owned)
            .collect();
        // The delta is drawn from self, so this never saturates.
        let skipped_count = self.len().saturating_sub(delta.len());
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
            // The batch runs in a savepoint: a failure rolls back to it, so the card-by-card
            // retry runs on a usable transaction.
            let mut savepoint = tx.begin().await?;
            let attempt = chunk.bulk_delta_upsert_with_tx(&mut savepoint).await;
            match &attempt {
                Ok(_) => savepoint.commit().await?,
                Err(_) => savepoint.rollback().await?,
            }
            match attempt {
                Ok((upserted, skipped)) => {
                    zervice_metrics
                        .add_upserted_count(zervice_metrics::count_column(upserted.len()));
                    zervice_metrics.add_skipped_count(zervice_metrics::count_column(skipped));
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

#[cfg(test)]
mod clash_tests {
    use super::clash_of;

    #[test]
    fn a_unique_violation_detail_reads_as_its_column_and_value() {
        assert_eq!(
            clash_of("Key (mtgo_id)=(777) already exists."),
            Some(("mtgo_id", "777"))
        );
        assert_eq!(
            clash_of("Key (id)=(5a7e0000-0000-0000-0000-000000000002) already exists."),
            Some(("id", "5a7e0000-0000-0000-0000-000000000002"))
        );
        assert_eq!(clash_of("Failing row contains (...)."), None);
    }
}
