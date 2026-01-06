use crate::domain::card::models::card_profile::CardProfile;
use crate::domain::card::models::helpers::SleeveScryfallData;
use crate::domain::card::models::scryfall_data::get_scryfall_data::ScryfallDataIds;
use crate::domain::card::models::{Card, create_card::CreateCardError};
use crate::domain::card::models::{
    scryfall_data::ScryfallData,
    sync_metrics::{ErrorMetrics, SyncMetrics},
};
use crate::outbound::sqlx::card::card_profile::DatabaseCardProfile;
use crate::outbound::sqlx::card::helpers::scryfall_data_fields::{
    BindCards, BindScryfallDataFields, bulk_upsert_conflict_fields, scryfall_data_fields,
};
use sqlx::QueryBuilder;
use sqlx::{PgTransaction, query_as};
use std::future::Future;

/// for use in error filtering below
const POSTGRES_TX_ABORT_MESSAGE: &str = "current transaction is aborted";

// ====================
//  validation helpers
// ====================

/// determines if a card is a token based on Scryfall layout field
fn is_token(scryfall_data: &ScryfallData) -> bool {
    scryfall_data.layout == "token"
}

/// determines if a card is a valid MTG commander
fn is_valid_commander(scryfall_data: &ScryfallData) -> bool {
    // check for special "can be your commander" text
    if let Some(text) = &scryfall_data.oracle_text
        && text.to_lowercase().contains("can be your commander")
    {
        return true;
    }

    let type_line_lower = scryfall_data.type_line.to_lowercase();

    // check legendary creature
    if type_line_lower.contains("legendary") && type_line_lower.contains("creature") {
        return true;
    }

    // check legendary vehicle/spacecraft with power/toughness
    if type_line_lower.contains("legendary")
        && (type_line_lower.contains("vehicle") || type_line_lower.contains("spacecraft"))
        && let (Some(power), Some(toughness)) = (&scryfall_data.power, &scryfall_data.toughness)
            && !power.is_empty()
            && !toughness.is_empty()
        {
            return true;
        }

    false
}

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
        let is_commander = is_valid_commander(&scryfall_data);
        let is_tok = is_token(&scryfall_data);
        let database_card_profile = query_as!(
            DatabaseCardProfile,
            "INSERT INTO card_profiles (scryfall_data_id, is_valid_commander, is_token)
             VALUES ($1, $2, $3)
             ON CONFLICT (scryfall_data_id)
             DO UPDATE SET updated_at = NOW(), is_valid_commander = EXCLUDED.is_valid_commander, is_token = EXCLUDED.is_token
             RETURNING id, scryfall_data_id, is_valid_commander, is_token, created_at, updated_at",
            scryfall_data_id,
            is_commander,
            is_tok
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
        let mut card_profile_query_builder = QueryBuilder::new(
            "INSERT INTO card_profiles (scryfall_data_id, is_valid_commander, is_token) VALUES",
        );
        for (i, scryfall_data) in database_scryfall_data.iter().enumerate() {
            if i > 0 {
                card_profile_query_builder.push(",");
            }
            let is_commander = is_valid_commander(scryfall_data);
            let is_tok = is_token(scryfall_data);
            card_profile_query_builder
                .push("(")
                .push_bind(scryfall_data.id)
                .push(",")
                .push_bind(is_commander)
                .push(",")
                .push_bind(is_tok)
                .push(")");
        }
        card_profile_query_builder
            .push(" ON CONFLICT (scryfall_data_id) DO UPDATE SET updated_at = NOW(), is_valid_commander = EXCLUDED.is_valid_commander, is_token = EXCLUDED.is_token ");
        card_profile_query_builder.push(" RETURNING id, scryfall_data_id, is_valid_commander, is_token, created_at, updated_at;");
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
