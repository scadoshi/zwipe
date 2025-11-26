use crate::domain::card::models::card_profile::CardProfile;
use crate::domain::card::models::helpers::SleeveScryfallData;
use crate::domain::card::models::scryfall_data::get_scryfall_data::ScryfallDataIds;
use crate::domain::card::models::{create_card::CreateCardError, Card};
use crate::domain::card::models::{
    scryfall_data::ScryfallData,
    sync_metrics::{ErrorMetrics, SyncMetrics},
};
use crate::outbound::sqlx::card::card_profile::DatabaseCardProfile;
use crate::outbound::sqlx::card::helpers::scryfall_data_fields::{
    bulk_upsert_conflict_fields, scryfall_data_fields, BindCards, BindScryfallDataFields,
};
use sqlx::QueryBuilder;
use sqlx::{query_as, PgTransaction};
use std::future::Future;

/// for use in error filtering below
const POSTGRES_TX_ABORT_MESSAGE: &str = "current transaction is aborted";

// ===========
//  insertion
// ===========

// below allows redundant operations within `CardRepository`
// without having to create new transactions
// these should **not** commit the transaction

/// single insert/update of `ScryfallData`
/// - also inserts/updates `CardProfile`
pub trait SingleUpsertWithTx
where
    Self: Sized,
{
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
        let scryfall_data: ScryfallData = sfd_qb
            .build_query_as::<ScryfallData>()
            .fetch_one(&mut **tx)
            .await?;
        let database_card_profile = query_as!(
            DatabaseCardProfile,
            "INSERT INTO card_profiles (scryfall_data_id) VALUES ($1)
            ON CONFLICT (scryfall_data_id)
            DO UPDATE SET updated_at = NOW()
            RETURNING id, scryfall_data_id, created_at, updated_at",
            scryfall_data_id
        )
        .fetch_one(&mut **tx)
        .await?;
        let card_profile: CardProfile = database_card_profile.into();
        let card = Card::new(card_profile, scryfall_data);
        Ok(card)
    }
}

/// bulk insert/update of `ScryfallData`
/// - also inserts/updates `CardProfile`
/// - includes no special batching
pub trait BulkUpsertWithTx
where
    Self: Sized,
{
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
        let database_scryfall_data: Vec<ScryfallData> = scryfall_data_query_builder
            .build_query_as::<ScryfallData>()
            .fetch_all(&mut **tx)
            .await?;
        let mut card_profile_query_builder =
            QueryBuilder::new("INSERT INTO card_profiles (scryfall_data_id) VALUES");
        for (i, id) in database_scryfall_data.iter().map(|x| x.id).enumerate() {
            if i > 0 {
                card_profile_query_builder.push(",");
            }
            card_profile_query_builder.push("(").push_bind(id).push(")");
        }
        card_profile_query_builder
            .push(" ON CONFLICT (scryfall_data_id) DO UPDATE SET updated_at = NOW() ");
        card_profile_query_builder.push(" RETURNING id, scryfall_data_id, created_at, updated_at;");
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

/// batch insert/update of `ScryfallData`
/// - also inserts/updates `CardProfile`
/// - uses `BulkUpsertWithTx` interally to perform batching
pub trait BatchUpsertWithTx
where
    Self: Sized,
{
    fn batch_upsert_with_tx(
        self,
        tx: &mut PgTransaction<'_>,
        batch_size: usize,
        sync_metrics: &mut SyncMetrics,
    ) -> impl Future<Output = Result<Vec<Card>, CreateCardError>> + Send;
}

/// inserts/updates `ScryfallData` card by card
/// - also inserts/updates `CardProfile`
/// - usually in the event of a `BulkUpsertWithTx` failure
async fn upsert_card_by_card(
    batch: &[ScryfallData],
    tx: &mut PgTransaction<'_>,
    sync_metrics: &mut SyncMetrics,
) {
    for card in batch {
        match card.single_upsert_with_tx(tx).await {
            Ok(_) => {
                sync_metrics.add_upserted_count(1);
            }
            Err(e) => {
                // ignore tx abort messages as they are never root cause
                if !e.to_string().contains(POSTGRES_TX_ABORT_MESSAGE) {
                    let error = ErrorMetrics::new(card.id, &card.name, e.to_string());
                    tracing::warn!("insertion failure => {}", error);
                    sync_metrics.add_error(error);
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
        sync_metrics: &mut SyncMetrics,
    ) -> Result<Vec<Card>, CreateCardError> {
        let mut cards: Vec<Card> = Vec::new();
        for chunk in self.chunks(batch_size) {
            match chunk.bulk_upsert_with_tx(tx).await {
                Ok(upserted) => {
                    let upserted_count = upserted.len();
                    cards.extend(upserted);
                    sync_metrics.add_upserted_count(upserted_count as i32);
                }
                Err(e) => {
                    tracing::warn!("batch failed with error: {:?}\nretrying card by card", e);
                    upsert_card_by_card(chunk, tx, sync_metrics).await;
                }
            }
        }
        Ok(cards)
    }
}

pub trait BulkDeltaUpsertWithTx
where
    Self: Sized,
{
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
        let existing: Vec<ScryfallData> =
            query_as("SELECT * FROM scryfall_data WHERE id = ANY($1)")
                .bind(ScryfallDataIds::from(self).ids())
                .fetch_all(&mut **tx)
                .await
                .map_err(|e| CreateCardError::GetScryfallData(e.into()))?;
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

pub trait BatchDeltaUpsertWithTx
where
    Self: Sized,
{
    fn batch_delta_upsert_with_tx(
        self,
        tx: &mut PgTransaction<'_>,
        batch_size: usize,
        sync_metrics: &mut SyncMetrics,
    ) -> impl Future<Output = Result<Vec<Card>, CreateCardError>> + Send;
}

impl BatchDeltaUpsertWithTx for &[ScryfallData] {
    async fn batch_delta_upsert_with_tx(
        self,
        tx: &mut PgTransaction<'_>,
        batch_size: usize,
        sync_metrics: &mut SyncMetrics,
    ) -> Result<Vec<Card>, CreateCardError> {
        let mut cards: Vec<Card> = Vec::new();
        for chunk in self.chunks(batch_size) {
            match chunk.bulk_delta_upsert_with_tx(tx).await {
                Ok((upserted, skipped)) => {
                    sync_metrics.add_upserted_count(upserted.len() as i32);
                    sync_metrics.add_skipped_count(skipped as i32);
                    cards.extend(upserted);
                }
                Err(e) => {
                    tracing::warn!("batch failed with error: {:?}\nretrying card by card", e);
                    upsert_card_by_card(chunk, tx, sync_metrics).await;
                }
            }
        }
        Ok(cards)
    }
}
