pub mod card_profile;
pub mod scryfall_data;
pub mod sync_metrics;

use crate::domain::card::models::card_profile::{
    CardProfile, GetCardProfile, GetCardProfileError, GetCardProfiles,
};
use crate::domain::card::models::scryfall_data::{GetScryfallDataError, SearchScryfallDataError};
use crate::domain::card::models::{
    Card, CreateCardError, GetCard, GetCardError, GetCards, SearchCard, SearchCardError, Sleeve,
};
use crate::outbound::sqlx::card::card_profile::DatabaseCardProfile;
use crate::outbound::sqlx::postgres::{IsConstraintViolation, Postgres as MyPostgres};
use crate::{
    domain::card::{
        models::{
            scryfall_data::ScryfallData,
            sync_metrics::{ErrorMetrics, SyncMetrics, SyncType},
        },
        ports::CardRepository,
    },
    outbound::sqlx::card::sync_metrics::DatabaseSyncMetrics,
};

use anyhow::Context;
use chrono::NaiveDateTime;
use sqlx::{query, query_as, query_scalar, PgTransaction, Postgres, Transaction};
use sqlx::{query_builder::Separated, QueryBuilder};
use std::{collections::HashSet, future::Future};
use uuid::Uuid;

// =========
//  helpers
// =========

/// for use in error filtering below
const POSTGRES_TX_ABORT_MESSAGE: &str = "current transaction is aborted";

/// scryfall card fields for use in query field tuples
const SCRYFALL_CARD_FIELDS: &str = r#"
    arena_id
    id
    lang
    mtgo_id
    mtgo_foil_id
    multiverse_ids
    tcgplayer_id
    tcgplayer_etched_id
    cardmarket_id
    object
    layout
    oracle_id
    prints_search_uri
    rulings_uri
    scryfall_uri
    uri
    all_parts
    card_faces
    cmc
    color_identity
    color_indicator
    colors
    defense
    edhrec_rank
    game_changer
    hand_modifier
    keywords
    legalities
    life_modifier
    loyalty
    mana_cost
    name
    oracle_text
    penny_rank
    power
    produced_mana
    reserved
    toughness
    type_line
    artist
    artist_ids
    attraction_lights
    booster
    border_color
    card_back_id
    collector_number
    content_warning
    digital
    finishes
    flavor_name
    flavor_text
    frame_effects
    frame
    full_art
    games
    highres_image
    illustration_id
    image_status
    image_uris
    oversized
    prices
    printed_name
    printed_text
    printed_type_line
    promo
    promo_types
    purchase_uris
    rarity
    related_uris
    released_at
    reprint
    scryfall_set_uri
    set_name
    set_search_uri
    set_type
    set_uri
    set
    set_id
    story_spotlight
    textless
    variation
    variation_of
    security_stamp
    watermark
    preview_previewed_at
    preview_source_uri
    preview_source
"#;

pub fn scryfall_data_fields() -> String {
    SCRYFALL_CARD_FIELDS
        .trim()
        .lines()
        .map(|x| x.trim().to_string())
        .collect::<Vec<String>>()
        .join(",")
}

pub fn scryfall_data_field_count() -> usize {
    SCRYFALL_CARD_FIELDS
        .trim()
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .count()
}

/// for pushing all scryfall data fields
/// onto a `QueryBuilder``

pub trait BindScryfallDataFields {
    fn bind_scryfall_fields(&mut self, card: &ScryfallData);
}

impl BindScryfallDataFields for QueryBuilder<'_, Postgres> {
    fn bind_scryfall_fields(&mut self, card: &ScryfallData) {
        self.push("(");
        // core card fields
        // cards have the following core properties
        self.push_bind(card.arena_id);
        self.push(", ");
        self.push_bind(card.id);
        self.push(", ");
        self.push_bind(card.lang.clone());
        self.push(", ");
        self.push_bind(card.mtgo_id);
        self.push(", ");
        self.push_bind(card.mtgo_foil_id);
        self.push(", ");
        self.push_bind(card.multiverse_ids.clone());
        self.push(", ");
        self.push_bind(card.tcgplayer_id);
        self.push(", ");
        self.push_bind(card.tcgplayer_etched_id);
        self.push(", ");
        self.push_bind(card.cardmarket_id);
        self.push(", ");
        self.push_bind(card.object.clone());
        self.push(", ");
        self.push_bind(card.layout.clone());
        self.push(", ");
        self.push_bind(card.oracle_id);
        self.push(", ");
        self.push_bind(card.prints_search_uri.clone());
        self.push(", ");
        self.push_bind(card.rulings_uri.clone());
        self.push(", ");
        self.push_bind(card.scryfall_uri.clone());
        self.push(", ");
        self.push_bind(card.uri.clone());
        self.push(", ");
        // gameplay fields
        // cards have the following properties relevant to the game rules
        self.push_bind(card.all_parts.clone());
        self.push(", ");
        self.push_bind(card.card_faces.clone());
        self.push(", ");
        self.push_bind(card.cmc);
        self.push(", ");
        self.push_bind(card.color_identity.clone());
        self.push(", ");
        self.push_bind(card.color_indicator.clone());
        self.push(", ");
        self.push_bind(card.colors.clone());
        self.push(", ");
        self.push_bind(card.defense.clone());
        self.push(", ");
        self.push_bind(card.edhrec_rank);
        self.push(", ");
        self.push_bind(card.game_changer);
        self.push(", ");
        self.push_bind(card.hand_modifier.clone());
        self.push(", ");
        self.push_bind(card.keywords.clone());
        self.push(", ");
        self.push_bind(card.legalities.clone());
        self.push(", ");
        self.push_bind(card.life_modifier.clone());
        self.push(", ");
        self.push_bind(card.loyalty.clone());
        self.push(", ");
        self.push_bind(card.mana_cost.clone());
        self.push(", ");
        self.push_bind(card.name.clone());
        self.push(", ");
        self.push_bind(card.oracle_text.clone());
        self.push(", ");
        self.push_bind(card.penny_rank);
        self.push(", ");
        self.push_bind(card.power.clone());
        self.push(", ");
        self.push_bind(card.produced_mana.clone());
        self.push(", ");
        self.push_bind(card.reserved);
        self.push(", ");
        self.push_bind(card.toughness.clone());
        self.push(", ");
        self.push_bind(card.type_line.clone());
        self.push(", ");
        // print fields
        // cards have the following properties unique to their particular re/print
        self.push_bind(card.artist.clone());
        self.push(", ");
        self.push_bind(card.artist_ids.clone());
        self.push(", ");
        self.push_bind(card.attraction_lights.clone());
        self.push(", ");
        self.push_bind(card.booster);
        self.push(", ");
        self.push_bind(card.border_color.clone());
        self.push(", ");
        self.push_bind(card.card_back_id);
        self.push(", ");
        self.push_bind(card.collector_number.clone());
        self.push(", ");
        self.push_bind(card.content_warning);
        self.push(", ");
        self.push_bind(card.digital);
        self.push(", ");
        self.push_bind(card.finishes.clone());
        self.push(", ");
        self.push_bind(card.flavor_name.clone());
        self.push(", ");
        self.push_bind(card.flavor_text.clone());
        self.push(", ");
        self.push_bind(card.frame_effects.clone());
        self.push(", ");
        self.push_bind(card.frame.clone());
        self.push(", ");
        self.push_bind(card.full_art);
        self.push(", ");
        self.push_bind(card.games.clone());
        self.push(", ");
        self.push_bind(card.highres_image);
        self.push(", ");
        self.push_bind(card.illustration_id);
        self.push(", ");
        self.push_bind(card.image_status.clone());
        self.push(", ");
        self.push_bind(card.image_uris.clone());
        self.push(", ");
        self.push_bind(card.oversized);
        self.push(", ");
        self.push_bind(card.prices.clone());
        self.push(", ");
        self.push_bind(card.printed_name.clone());
        self.push(", ");
        self.push_bind(card.printed_text.clone());
        self.push(", ");
        self.push_bind(card.printed_type_line.clone());
        self.push(", ");
        self.push_bind(card.promo);
        self.push(", ");
        self.push_bind(card.promo_types.clone());
        self.push(", ");
        self.push_bind(card.purchase_uris.clone());
        self.push(", ");
        self.push_bind(card.rarity.clone());
        self.push(", ");
        self.push_bind(card.related_uris.clone());
        self.push(", ");
        self.push_bind(card.released_at);
        self.push(", ");
        self.push_bind(card.reprint);
        self.push(", ");
        self.push_bind(card.scryfall_set_uri.clone());
        self.push(", ");
        self.push_bind(card.set_name.clone());
        self.push(", ");
        self.push_bind(card.set_search_uri.clone());
        self.push(", ");
        self.push_bind(card.set_type.clone());
        self.push(", ");
        self.push_bind(card.set_uri.clone());
        self.push(", ");
        self.push_bind(card.set.clone());
        self.push(", ");
        self.push_bind(card.set_id);
        self.push(", ");
        self.push_bind(card.story_spotlight);
        self.push(", ");
        self.push_bind(card.textless);
        self.push(", ");
        self.push_bind(card.variation);
        self.push(", ");
        self.push_bind(card.variation_of);
        self.push(", ");
        self.push_bind(card.security_stamp.clone());
        self.push(", ");
        self.push_bind(card.watermark.clone());
        self.push(", ");
        self.push_bind(card.preview_previewed_at);
        self.push(", ");
        self.push_bind(card.preview_source_uri.clone());
        self.push(", ");
        self.push_bind(card.preview_source.clone());
        self.push(")");
    }
}

// =========
//  errors
// =========

impl From<sqlx::Error> for CreateCardError {
    fn from(value: sqlx::Error) -> Self {
        if value.is_unique_constraint_violation() {
            return CreateCardError::UniqueConstraintViolation(value.into());
        }
        CreateCardError::Database(value.into())
    }
}

impl From<sqlx::Error> for GetScryfallDataError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => GetScryfallDataError::NotFound,
            e => GetScryfallDataError::Database(e.into()),
        }
    }
}

impl From<sqlx::Error> for GetCardProfileError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => Self::NotFound,
            e => Self::Database(e.into()),
        }
    }
}

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

trait BindToSeparator {
    fn bind_to(&self, qb: &mut QueryBuilder<'_, Postgres>);
}

impl BindToSeparator for Vec<ScryfallData> {
    fn bind_to(&self, qb: &mut QueryBuilder<'_, Postgres>) {
        let mut needs_comma = false;
        for card in self {
            if needs_comma {
                qb.push(", ");
            }
            qb.bind_scryfall_fields(card);
            needs_comma = true;
        }
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

// tx commits should be handled at this level rather than above
impl CardRepository for MyPostgres {
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
        request: &SearchCard,
    ) -> Result<Vec<ScryfallData>, SearchScryfallDataError> {
        let mut qb: QueryBuilder<'_, Postgres> = QueryBuilder::new("SELECT * FROM scryfall_data");
        let mut sep: Separated<Postgres, &'static str> = qb.separated(" AND ");

        let mut n = 1;

        if let Some(name) = &request.name {
            sep.push(format!("name ILIKE ${}", n));
            sep.push_bind_unseparated(format!("%{}%", name));
            n += 1;
        }
        if let Some(type_line) = &request.type_line {
            sep.push(format!("type_line ILIKE ${}", n));
            sep.push_bind_unseparated(format!("%{}%", type_line));
            n += 1;
        }
        if let Some(set) = &request.set {
            sep.push(format!("set ILIKE ${}", n));
            sep.push_bind_unseparated(format!("%{}%", set));
            n += 1;
        }
        if let Some(rarity) = &request.rarity {
            sep.push(format!("rarity ILIKE ${}", n));
            sep.push_bind_unseparated(format!("%{}%", rarity));
            n += 1;
        }
        if let Some(cmc) = request.cmc {
            sep.push(format!("cmc = ${}", n));
            sep.push_bind_unseparated(cmc);
            n += 1;
        }
        if let Some(color_identity) = &request.color_identity {
            sep.push(format!("color_identity && ${}", n));
            sep.push_bind_unseparated(color_identity);
            n += 1;
        }
        if let Some(oracle_text) = &request.oracle_text {
            sep.push(format!("oracle_text  ILIKE ${}", n));
            sep.push_bind_unseparated(format!("%{}%", oracle_text));
            n += 1;
        }
        if let Some(limit) = request.limit {
            qb.push(format!(" LIMIT ${}", n));
            qb.push_bind(limit as i32);
            n += 1;
        }
        if let Some(offset) = request.offset {
            qb.push(format!(" OFFSET ${}", n));
            qb.push_bind(offset as i32);
        }

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

    async fn search_cards(&self, request: &SearchCard) -> Result<Vec<Card>, SearchCardError> {
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
