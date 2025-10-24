use crate::domain::card::models::card_profile::CardProfile;
use crate::domain::card::models::helpers::SleeveScryfallData;
use crate::domain::card::models::{create_card::CreateCardError, Card};
use crate::domain::card::models::{
    scryfall_data::ScryfallData,
    sync_metrics::{ErrorMetrics, SyncMetrics},
};
use crate::outbound::sqlx::card::card_profile::DatabaseCardProfile;
use crate::outbound::sqlx::card::helpers::scryfall_data_fields::{
    scryfall_data_fields, BindScryfallDataFields, BindToSeparator,
};
use sqlx::QueryBuilder;
use sqlx::{query_as, PgTransaction, Postgres, Transaction};
use std::{collections::HashSet, future::Future};
use uuid::Uuid;

/// for use in error filtering below
const POSTGRES_TX_ABORT_MESSAGE: &str = "current transaction is aborted";

// ===========
//  insertion
// ===========

// below allows redundant operations within `CardRepository`
// without having to create new transactions
// these should **not** commit the transaction

/// basic insertions like a single card
/// or multiple cards with no special batching
///
/// this takes a transaction and mutates
/// leaving commitment for higher level functions
pub trait InsertCardWithTx
where
    Self: Sized,
{
    fn insert_with_tx(
        &self,
        tx: &mut PgTransaction<'_>,
    ) -> impl Future<Output = Result<Card, CreateCardError>> + Send;
}

/// for inserting a single card given a transaction
impl InsertCardWithTx for ScryfallData {
    async fn insert_with_tx(&self, tx: &mut PgTransaction<'_>) -> Result<Card, CreateCardError> {
        let scryfall_data_id = self.id.clone();
        let mut sfd_qb = QueryBuilder::new("INSERT INTO scryfall_data (");
        sfd_qb.push(scryfall_data_fields()).push(") VALUES ");
        sfd_qb.bind_scryfall_fields(self);
        sfd_qb.push(" RETURNING *");
        let scryfall_data: ScryfallData = sfd_qb
            .build_query_as::<ScryfallData>()
            .fetch_one(&mut **tx)
            .await?;

        let database_card_profile = query_as!(
            DatabaseCardProfile,
            "INSERT INTO card_profiles (scryfall_data_id) VALUES ($1) RETURNING id, scryfall_data_id",
            scryfall_data_id
        )
        .fetch_one(&mut **tx)
        .await?;

        let card_profile: CardProfile = database_card_profile.into();

        let card = Card::new(card_profile, scryfall_data);
        Ok(card)
    }
}

pub trait InsertCardsWithTx
where
    Self: Sized,
{
    fn insert_with_tx(
        &self,
        tx: &mut PgTransaction<'_>,
    ) -> impl Future<Output = Result<Vec<Card>, CreateCardError>> + Send;
}

/// for inserting multiple cards given a transaction
impl InsertCardsWithTx for &[ScryfallData] {
    async fn insert_with_tx(
        &self,
        tx: &mut PgTransaction<'_>,
    ) -> Result<Vec<Card>, CreateCardError> {
        let scryfall_data_ids: HashSet<Uuid> = self.iter().map(|sfd| sfd.id.to_owned()).collect();

        let mut sfd_qb = QueryBuilder::new("INSERT INTO scryfall_data (");
        sfd_qb.push(scryfall_data_fields()).push(") VALUES ");

        let mut sfd: Vec<ScryfallData> = self.to_vec();
        sfd.dedup_by_key(|sfd| sfd.id.clone());
        sfd.bind_to(&mut sfd_qb);
        sfd_qb.push(" RETURNING *;");

        let sfd: Vec<ScryfallData> = sfd_qb
            .build_query_as::<ScryfallData>()
            .fetch_all(&mut **tx)
            .await?;

        let mut cp_qb = QueryBuilder::new("INSERT INTO card_profiles (scryfall_data_id) VALUES");

        let mut needs_comma = false;
        for id in scryfall_data_ids {
            if needs_comma {
                cp_qb.push(",");
            }
            cp_qb.push("(");
            cp_qb.push_bind(id);
            cp_qb.push(")");
            needs_comma = true;
        }

        cp_qb.push(" RETURNING id, scryfall_data_id");

        let cps: Vec<CardProfile> = cp_qb
            .build_query_as::<DatabaseCardProfile>()
            .fetch_all(&mut **tx)
            .await?
            .into_iter()
            .map(|dcp| dcp.into())
            .collect();

        let cards: Vec<Card> = sfd.sleeve(cps);

        Ok(cards)
    }
}

/// for batch insertions of multiple cards
///
/// this takes a transaction and mutates
/// leaving commitment for higher level functions
pub trait BatchInsertWithTx
where
    Self: Sized,
{
    fn batch_insert_with_tx(
        self,
        tx: &mut PgTransaction<'_>,
        batch_size: usize,
        sync_metrics: &mut SyncMetrics,
    ) -> impl Future<Output = Result<Vec<Card>, CreateCardError>> + Send;
}

/// for inserting cards in a batch card by card
/// usually in the event that a batch insert fails
///
/// impl of BatchInsertWithTransaction uses this internally
async fn insert_card_by_card(
    batch: Vec<ScryfallData>,
    tx: &mut Transaction<'_, Postgres>,
    sync_metrics: &mut SyncMetrics,
) {
    for card in batch {
        let card_name = card.name.clone();
        let card_id = card.id.clone();

        match card.insert_with_tx(tx).await {
            Ok(_) => sync_metrics.add_imported(1),
            Err(e) => {
                // ignore tx abort messages as they are never root cause
                if !e.to_string().contains(POSTGRES_TX_ABORT_MESSAGE) {
                    let error = ErrorMetrics::new(card_id, &card_name, &e.to_string());
                    tracing::warn!("insertion failure => {}", error);
                    sync_metrics.add_error(error);
                }
            }
        }
    }
}

impl BatchInsertWithTx for &[ScryfallData] {
    async fn batch_insert_with_tx(
        self,
        tx: &mut PgTransaction<'_>,
        batch_size: usize,
        sync_metrics: &mut SyncMetrics,
    ) -> Result<Vec<Card>, CreateCardError> {
        let mut cards: Vec<Card> = Vec::new();

        for chunk in self.chunks(batch_size) {
            match chunk.insert_with_tx(tx).await {
                Ok(inserted) => {
                    let inserted_count = inserted.len();
                    cards.extend(inserted);
                    sync_metrics.add_imported(inserted_count as i32);
                }
                Err(e) => {
                    tracing::warn!("batch failed with error: {:?}\nretrying card by card", e);
                    insert_card_by_card(chunk.to_owned(), tx, sync_metrics).await;
                }
            }
        }

        Ok(cards)
    }
}
