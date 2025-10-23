pub mod card_profile;
pub mod error;
pub mod helpers;
pub mod scryfall_data;
pub mod sync_metrics;

use crate::domain::card::models::card_profile::{
    get_card_profile::{GetCardProfile, GetCardProfileError, GetCardProfiles},
    CardProfile,
};
use crate::domain::card::models::scryfall_data::{GetScryfallDataError, SearchScryfallDataError};
use crate::domain::card::models::{
    create_card::CreateCardError,
    get_card::{GetCard, GetCardError, GetCards},
    helpers::Sleeve,
    search_card::{SearchCards, SearchCardsError},
    Card,
};
use crate::outbound::sqlx::card::card_profile::DatabaseCardProfile;
use crate::outbound::sqlx::card::helpers::insert_card::{
    BatchInsertWithTx, InsertCardWithTx, InsertCardsWithTx,
};
use crate::outbound::sqlx::postgres::Postgres as MyPostgres;
use crate::{
    domain::card::{
        models::{
            scryfall_data::ScryfallData,
            sync_metrics::{SyncMetrics, SyncType},
        },
        ports::CardRepository,
    },
    outbound::sqlx::card::sync_metrics::DatabaseSyncMetrics,
};

use anyhow::Context;
use chrono::NaiveDateTime;
use sqlx::{query, query_as, query_scalar, Execute, Postgres};
use sqlx::{query_builder::Separated, QueryBuilder};
use uuid::Uuid;

impl CardRepository for MyPostgres {
    // ========
    //  create
    // ========
    async fn insert(&self, scryfall_data: &ScryfallData) -> Result<Card, CreateCardError> {
        let mut tx = self.pool.begin().await?;
        let card = scryfall_data.insert_with_tx(&mut tx).await?;
        tx.commit().await?;
        Ok(card)
    }

    async fn bulk_insert(
        &self,
        scryfall_data: &[ScryfallData],
    ) -> Result<Vec<Card>, CreateCardError> {
        let mut tx = self.pool.begin().await?;
        let cards = scryfall_data.insert_with_tx(&mut tx).await?;
        tx.commit().await?;
        Ok(cards)
    }

    async fn batch_insert(
        &self,
        cards: &[ScryfallData],
        batch_size: usize,
        sync_metrics: &mut SyncMetrics,
    ) -> Result<Vec<Card>, CreateCardError> {
        let mut tx = self.pool.begin().await?;
        let cards = cards
            .batch_insert_with_tx(&mut tx, batch_size, sync_metrics)
            .await?;
        tx.commit().await?;
        Ok(cards)
    }

    async fn batch_insert_if_not_exists(
        &self,
        scryfall_data: &[ScryfallData],
        batch_size: usize,
        sync_metrics: &mut SyncMetrics,
    ) -> Result<Vec<Card>, CreateCardError> {
        tracing::info!("initiating batch insert if not exists process");
        tracing::info!("received {} cards", scryfall_data.len());
        let mut tx = self.pool.begin().await?;

        let existing_ids: Vec<Uuid> = query_scalar!("SELECT id FROM scryfall_data")
            .fetch_all(&self.pool)
            .await?;

        tracing::info!(
            "skipping {} cards because database already has them",
            existing_ids.len()
        );
        sync_metrics.set_skipped(existing_ids.len() as i32);

        let new_data: Vec<ScryfallData> = scryfall_data
            .iter()
            .filter(|sfd| !existing_ids.contains(&sfd.id))
            .map(|sfd| sfd.clone())
            .collect();

        if new_data.is_empty() {
            tracing::info!("no new cards to import so database is up to date");
            return Ok(Vec::new());
        }

        tracing::info!("importing {} new cards", new_data.len());
        let cards: Vec<Card> = new_data
            .batch_insert_with_tx(&mut tx, batch_size, sync_metrics)
            .await?;

        tx.commit().await?;

        Ok(cards)
    }

    async fn delete_if_exists_and_batch_insert(
        &self,
        cards: &[ScryfallData],
        batch_size: usize,
        sync_metrics: &mut SyncMetrics,
    ) -> Result<Vec<Card>, CreateCardError> {
        tracing::info!("initiating delete if exists and insert process");
        tracing::info!("received {} cards", cards.len());

        let mut tx = self.pool.begin().await?;

        // extract ids for deletion
        let card_ids: Vec<Uuid> = cards.iter().map(|c| c.id).collect();

        tracing::info!("deleting {} cards", card_ids.len());

        // delete the cards (card_profile cascade cascades)
        query!("DELETE FROM scryfall_data WHERE id = ANY($1)", &card_ids)
            .execute(&mut *tx)
            .await?;

        tracing::info!("importing {} cards", cards.len());

        let cards: Vec<Card> = cards
            .batch_insert_with_tx(&mut tx, batch_size, sync_metrics)
            .await?;

        tx.commit().await?;

        Ok(cards)
    }

    async fn record_sync_metrics(
        &self,
        sync_metrics: &SyncMetrics,
    ) -> Result<SyncMetrics, anyhow::Error> {
        let mut tx = self.pool.begin().await?;
        let query_sql = "INSERT INTO scryfall_data_sync_metrics".to_string()
         + " (sync_type, started_at, ended_at, duration_in_seconds, status, received, imported, skipped, error_count, errors)"
         + " VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING *";
        let database_sync_metrics: DatabaseSyncMetrics = query_as(&query_sql)
            .bind(sync_metrics.sync_type().to_string())
            .bind(sync_metrics.started_at())
            .bind(sync_metrics.ended_at())
            .bind(sync_metrics.duration_in_seconds())
            .bind(sync_metrics.status().to_string())
            .bind(sync_metrics.received())
            .bind(sync_metrics.imported())
            .bind(sync_metrics.skipped())
            .bind(sync_metrics.error_count())
            .bind(sync_metrics.errors())
            .fetch_one(&mut *tx)
            .await?;
        let sync_metrics: SyncMetrics = database_sync_metrics.try_into()?;
        tx.commit().await?;
        Ok(sync_metrics)
    }

    // =====
    //  get
    // =====
    async fn get_scryfall_data(
        &self,
        request: &GetCard,
    ) -> Result<ScryfallData, GetScryfallDataError> {
        let scryfall_data: ScryfallData = query_as("SELECT * FROM scryfall_data WHERE id = $1")
            .bind(request.card_profile_id())
            .fetch_one(&self.pool)
            .await?;

        Ok(scryfall_data)
    }

    async fn get_multiple_scryfall_data(
        &self,
        request: &GetCards,
    ) -> Result<Vec<ScryfallData>, GetScryfallDataError> {
        let scryfall_data: Vec<ScryfallData> =
            query_as("SELECT * FROM scryfall_data WHERE id = ANY($1)")
                .bind(request.ids())
                .fetch_all(&self.pool)
                .await?;

        Ok(scryfall_data)
    }

    async fn search_scryfall_data(
        &self,
        request: &SearchCards,
    ) -> Result<Vec<ScryfallData>, SearchScryfallDataError> {
        let mut qb: QueryBuilder<'_, Postgres> =
            QueryBuilder::new("SELECT * FROM scryfall_data WHERE ");
        let mut sep: Separated<Postgres, &'static str> = qb.separated(" AND ");

        if let Some(name) = &request.name {
            sep.push("name ILIKE ");
            sep.push_bind_unseparated(format!("%{}%", name));
        }
        if let Some(type_line) = &request.type_line {
            sep.push("type_line ILIKE ");
            sep.push_bind_unseparated(format!("%{}%", type_line));
        }
        if let Some(set) = &request.set {
            sep.push("set ILIKE ");
            sep.push_bind_unseparated(format!("%{}%", set));
        }
        if let Some(rarity) = &request.rarity {
            sep.push("rarity ILIKE ");
            sep.push_bind_unseparated(format!("%{}%", rarity));
        }
        if let Some(cmc) = request.cmc {
            sep.push("cmc = ");
            sep.push_bind_unseparated(cmc);
        }
        if let Some(cmc_range) = request.cmc_range {
            let lower = cmc_range.0.min(cmc_range.1);
            let higher = cmc_range.0.max(cmc_range.1);
            sep.push("cmc between ");
            sep.push_bind_unseparated(lower);
            sep.push("and ");
            sep.push_bind_unseparated(higher);
        }
        if let Some(power) = request.power {
            sep.push("power ~ '^\\d+$' AND CAST(power AS INT) = ");
            sep.push_bind_unseparated(power);
        }
        if let Some(power_range) = request.power_range {
            let lower = power_range.0.min(power_range.1);
            let higher = power_range.0.max(power_range.1);
            sep.push("power ~ '^\\d+$' AND CAST(power AS INT) between ");
            sep.push_bind_unseparated(lower);
            sep.push("and ");
            sep.push_bind_unseparated(higher);
        }
        if let Some(toughness) = request.toughness {
            sep.push("toughness ~ '^\\d+$' AND CAST(toughness AS INT) = ");
            sep.push_bind_unseparated(toughness);
        }
        if let Some(toughness_range) = request.toughness_range {
            let lower = toughness_range.0.min(toughness_range.1);
            let higher = toughness_range.0.max(toughness_range.1);
            sep.push("toughness ~ '^\\d+$' AND CAST(toughness AS INT) between ");
            sep.push_bind_unseparated(lower);
            sep.push("and ");
            sep.push_bind_unseparated(higher);
        }
        if let Some(colors) = &request.color_identity {
            sep.push("color_identity @> ");
            sep.push_bind_unseparated(colors);
            sep.push("AND color_identity <@ ");
            sep.push_bind_unseparated(colors);
        }
        if let Some(colors) = &request.color_identity_contains {
            sep.push("color_identity && ");
            sep.push_bind_unseparated(colors);
        }
        if let Some(oracle_text) = &request.oracle_text {
            sep.push("oracle_text ILIKE ");
            sep.push_bind_unseparated(format!("%{}%", oracle_text));
        }
        if let Some(limit) = request.limit {
            qb.push(" LIMIT ");
            qb.push_bind(limit as i32);
        }
        if let Some(offset) = request.offset {
            qb.push(" OFFSET ");
            qb.push_bind(offset as i32);
        }

        tracing::info!("{}", qb.sql());

        let scryfall_data: Vec<ScryfallData> = qb.build_query_as().fetch_all(&self.pool).await?;

        Ok(scryfall_data)
    }

    async fn get_card(&self, request: &GetCard) -> Result<Card, GetCardError> {
        let scryfall_data = self.get_scryfall_data(request).await?;
        let get_card_profile = GetCardProfile::from(&scryfall_data);
        let card_profile = self.get_card_profile(&get_card_profile).await?;
        let card = Card::new(card_profile, scryfall_data);
        Ok(card)
    }

    async fn get_cards(&self, request: &GetCards) -> Result<Vec<Card>, GetCardError> {
        let scryfall_data = self.get_multiple_scryfall_data(request).await?;
        let get_card_profiles = GetCardProfiles::from(scryfall_data.as_slice());
        let card_profiles = self.get_card_profiles(&get_card_profiles).await?;
        let cards = scryfall_data.sleeve(card_profiles);
        Ok(cards)
    }

    async fn search_cards(&self, request: &SearchCards) -> Result<Vec<Card>, SearchCardsError> {
        let scryfall_data = self.search_scryfall_data(request).await?;
        let get_card_profiles = GetCardProfiles::from(scryfall_data.as_slice());
        let card_profiles = self.get_card_profiles(&get_card_profiles).await?;
        let cards = scryfall_data.sleeve(card_profiles);
        Ok(cards)
    }

    async fn get_card_profile(
        &self,
        request: &GetCardProfile,
    ) -> Result<CardProfile, GetCardProfileError> {
        let card_profile: CardProfile = query_as!(
            DatabaseCardProfile,
            "SELECT id, scryfall_data_id FROM card_profiles WHERE id = $1",
            request.id()
        )
        .fetch_one(&self.pool)
        .await?
        .into();

        Ok(card_profile)
    }

    async fn get_card_profiles(
        &self,
        request: &GetCardProfiles,
    ) -> Result<Vec<CardProfile>, GetCardProfileError> {
        let card_profiles: Vec<CardProfile> = query_as!(
            DatabaseCardProfile,
            "SELECT id, scryfall_data_id FROM card_profiles WHERE id = ANY($1)",
            request.ids()
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|dcp| dcp.into())
        .collect();

        Ok(card_profiles)
    }

    async fn get_last_sync_date(
        &self,
        sync_type: SyncType,
    ) -> anyhow::Result<Option<NaiveDateTime>> {
        let last_sync_date: Option<NaiveDateTime> = query_scalar!(
            "SELECT started_at FROM scryfall_data_sync_metrics WHERE sync_type = $1 ORDER BY started_at DESC LIMIT 1",
            sync_type.to_string()
        )
        .fetch_optional(&self.pool)
        .await
        .context("failed to get last sync date")?;

        Ok(last_sync_date)
    }
}
