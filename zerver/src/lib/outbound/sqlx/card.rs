pub mod card_profile;
pub mod scryfall_card;
pub mod sync_metrics;

use crate::domain::card::models::{
    CardProfile, CreateCardError, GetCardError, GetCardProfileError, GetCardProfileRequest,
    GetCardProfilesRequest, GetCardRequest, GetCardsRequest, SearchCardError,
};
use crate::outbound::sqlx::card::card_profile::{DatabaseCardProfile, ToCardProfileError};
use crate::outbound::sqlx::postgres::{IsConstraintViolation, Postgres as MyPostgres};
use crate::{
    domain::card::{
        models::{
            scryfall_card::ScryfallCard,
            sync_metrics::{ErrorMetrics, SyncMetrics, SyncType},
            SearchCardRequest,
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

pub fn scryfall_card_fields() -> String {
    SCRYFALL_CARD_FIELDS
        .trim()
        .lines()
        .map(|x| x.trim().to_string())
        .collect::<Vec<String>>()
        .join(",")
}

pub fn scryfall_card_field_count() -> usize {
    SCRYFALL_CARD_FIELDS
        .trim()
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .count()
}

/// for pushing all of the card fields
/// onto a `QueryBuilder``
macro_rules! bind_scryfall_card_fields {
    ($qb:expr, $card:expr) => {
        $qb.push("(");
        let mut inner_sep = $qb.separated(", ");
        inner_sep
            // core card fields
            // cards have the following core properties
            .push_bind($card.arena_id)
            .push_bind($card.id)
            .push_bind($card.lang)
            .push_bind($card.mtgo_id)
            .push_bind($card.mtgo_foil_id)
            .push_bind($card.multiverse_ids)
            .push_bind($card.tcgplayer_id)
            .push_bind($card.tcgplayer_etched_id)
            .push_bind($card.cardmarket_id)
            .push_bind($card.object)
            .push_bind($card.layout)
            .push_bind($card.oracle_id)
            .push_bind($card.prints_search_uri)
            .push_bind($card.rulings_uri)
            .push_bind($card.scryfall_uri)
            .push_bind($card.uri)
            // gameplay fields
            // cards have the following properties relevant to the game rules
            .push_bind($card.all_parts)
            .push_bind($card.card_faces)
            .push_bind($card.cmc)
            .push_bind($card.color_identity)
            .push_bind($card.color_indicator)
            .push_bind($card.colors)
            .push_bind($card.defense)
            .push_bind($card.edhrec_rank)
            .push_bind($card.game_changer)
            .push_bind($card.hand_modifier)
            .push_bind($card.keywords)
            .push_bind($card.legalities)
            .push_bind($card.life_modifier)
            .push_bind($card.loyalty)
            .push_bind($card.mana_cost)
            .push_bind($card.name)
            .push_bind($card.oracle_text)
            .push_bind($card.penny_rank)
            .push_bind($card.power)
            .push_bind($card.produced_mana)
            .push_bind($card.reserved)
            .push_bind($card.toughness)
            .push_bind($card.type_line)
            // print fields
            // cards have the following properties unique to their particular re/print
            .push_bind($card.artist)
            .push_bind($card.artist_ids)
            .push_bind($card.attraction_lights)
            .push_bind($card.booster)
            .push_bind($card.border_color)
            .push_bind($card.card_back_id)
            .push_bind($card.collector_number)
            .push_bind($card.content_warning)
            .push_bind($card.digital)
            .push_bind($card.finishes)
            .push_bind($card.flavor_name)
            .push_bind($card.flavor_text)
            .push_bind($card.frame_effects)
            .push_bind($card.frame)
            .push_bind($card.full_art)
            .push_bind($card.games)
            .push_bind($card.highres_image)
            .push_bind($card.illustration_id)
            .push_bind($card.image_status)
            .push_bind($card.image_uris)
            .push_bind($card.oversized)
            .push_bind($card.prices)
            .push_bind($card.printed_name)
            .push_bind($card.printed_text)
            .push_bind($card.printed_type_line)
            .push_bind($card.promo)
            .push_bind($card.promo_types)
            .push_bind($card.purchase_uris)
            .push_bind($card.rarity)
            .push_bind($card.related_uris)
            .push_bind($card.released_at)
            .push_bind($card.reprint)
            .push_bind($card.scryfall_set_uri)
            .push_bind($card.set_name)
            .push_bind($card.set_search_uri)
            .push_bind($card.set_type)
            .push_bind($card.set_uri)
            .push_bind($card.set)
            .push_bind($card.set_id)
            .push_bind($card.story_spotlight)
            .push_bind($card.textless)
            .push_bind($card.variation)
            .push_bind($card.variation_of)
            .push_bind($card.security_stamp)
            .push_bind($card.watermark)
            .push_bind($card.preview_previewed_at)
            .push_bind($card.preview_source_uri)
            .push_bind($card.preview_source);
        $qb.push(")");
    };
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

impl From<sqlx::Error> for GetCardError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => GetCardError::NotFound,
            e => GetCardError::Database(e.into()),
        }
    }
}

impl From<sqlx::Error> for SearchCardError {
    fn from(value: sqlx::Error) -> Self {
        SearchCardError::Database(value.into())
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
pub trait InsertWithTransaction
where
    Self: Sized,
{
    fn insert_with_tx(
        self,
        tx: &mut PgTransaction<'_>,
    ) -> impl Future<Output = Result<Self, CreateCardError>> + Send;
}

/// for inserting a single card given a transaction
impl InsertWithTransaction for ScryfallCard {
    async fn insert_with_tx(self, tx: &mut PgTransaction<'_>) -> Result<Self, CreateCardError> {
        let scryfall_card_id = self.id.clone();

        let mut qb = QueryBuilder::new("INSERT INTO scryfall_cards (");
        qb.push(scryfall_card_fields()).push(") VALUES ");
        bind_scryfall_card_fields!(qb, self);
        qb.push(" RETURNING *");
        let query_as = qb.build_query_as::<ScryfallCard>();
        let card: ScryfallCard = query_as.fetch_one(&mut **tx).await?;

        query("INSERT INTO card_profiles (scryfall_card_id) VALUES ($1)")
            .bind(scryfall_card_id)
            .execute(&mut **tx)
            .await?;

        Ok(card)
    }
}

trait BindToSeparator {
    fn bind_to(self, qb: &mut QueryBuilder<'_, Postgres>);
}

impl BindToSeparator for Vec<ScryfallCard> {
    fn bind_to(self, qb: &mut QueryBuilder<'_, Postgres>) {
        let mut needs_comma = false;
        for card in self {
            if needs_comma {
                qb.push(", ");
            }
            bind_scryfall_card_fields!(qb, card);
            needs_comma = true;
        }
    }
}

/// for inserting multiple cards given a transaction
impl InsertWithTransaction for Vec<ScryfallCard> {
    async fn insert_with_tx(self, tx: &mut PgTransaction<'_>) -> Result<Self, CreateCardError> {
        let card_ids: HashSet<Uuid> = self.iter().map(|x| x.id.to_owned()).collect();

        let mut qb = QueryBuilder::new("INSERT INTO scryfall_cards (");
        qb.push(scryfall_card_fields()).push(") VALUES ");

        self.bind_to(&mut qb);

        let query_as = qb.build_query_as::<ScryfallCard>();
        let cards: Vec<ScryfallCard> = query_as.fetch_all(&mut **tx).await?;

        let mut qb = QueryBuilder::new("INSERT INTO card_profiles (scryfall_card_id) VALUES");

        let mut one_done = false;
        for id in card_ids {
            if one_done {
                qb.push(",");
            }
            qb.push("(");
            qb.push_bind(id);
            qb.push(")");
            one_done = true;
        }

        qb.build().execute(&mut **tx).await?;

        Ok(cards)
    }
}

/// for batch insertions of multiple cards
///
/// this takes a transaction and mutates
/// leaving commitment for higher level functions
pub trait BatchInsertWithTransaction
where
    Self: Sized,
{
    fn batch_insert_with_tx(
        self,
        tx: &mut PgTransaction<'_>,
        batch_size: usize,
        sync_metrics: &mut SyncMetrics,
    ) -> impl Future<Output = Result<Self, CreateCardError>> + Send;
}

/// for inserting cards in a batch card by card
/// usually in the event that a batch insert fails
///
/// impl of BatchInsertWithTransaction uses this internally
async fn insert_card_by_card(
    batch: Vec<ScryfallCard>,
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

impl BatchInsertWithTransaction for Vec<ScryfallCard> {
    async fn batch_insert_with_tx(
        self,
        tx: &mut PgTransaction<'_>,
        batch_size: usize,
        sync_metrics: &mut SyncMetrics,
    ) -> Result<Self, CreateCardError> {
        let mut cards: Vec<ScryfallCard> = Vec::new();
        for chunk in self.chunks(batch_size) {
            match chunk.to_owned().insert_with_tx(tx).await {
                Ok(inserted) => {
                    let inserted_count = inserted.len() as i32;
                    cards.extend(inserted);
                    sync_metrics.add_imported(inserted_count);
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
    async fn insert(&self, card: ScryfallCard) -> Result<ScryfallCard, CreateCardError> {
        let mut tx = self.pool.begin().await?;
        let card = card.insert_with_tx(&mut tx).await?;
        tx.commit().await?;
        Ok(card)
    }
    async fn bulk_insert(
        &self,
        cards: Vec<ScryfallCard>,
    ) -> Result<Vec<ScryfallCard>, CreateCardError> {
        let mut tx = self.pool.begin().await?;
        let cards = cards.insert_with_tx(&mut tx).await?;
        tx.commit().await?;
        Ok(cards)
    }
    async fn batch_insert(
        &self,
        cards: Vec<ScryfallCard>,
        batch_size: usize,
        sync_metrics: &mut SyncMetrics,
    ) -> Result<Vec<ScryfallCard>, CreateCardError> {
        let mut tx = self.pool.begin().await?;
        let cards = cards
            .batch_insert_with_tx(&mut tx, batch_size, sync_metrics)
            .await?;
        tx.commit().await?;
        Ok(cards)
    }
    async fn batch_insert_if_not_exists(
        &self,
        cards: Vec<ScryfallCard>,
        batch_size: usize,
        sync_metrics: &mut SyncMetrics,
    ) -> Result<Vec<ScryfallCard>, CreateCardError> {
        tracing::info!("initiating batch insert if not exists process");
        tracing::info!("received {} cards", cards.len());
        let mut tx = self.pool.begin().await?;
        let existing_ids: Vec<Uuid> = query_scalar("SELECT id FROM scryfall_cards")
            .fetch_all(&self.pool)
            .await?;
        tracing::info!(
            "skipping {} cards because database already has them",
            existing_ids.len()
        );
        sync_metrics.set_skipped(existing_ids.len() as i32);
        let new_cards: Vec<ScryfallCard> = cards
            .into_iter()
            .filter(|x| !existing_ids.contains(&x.id))
            .collect();
        if new_cards.is_empty() {
            tracing::info!("no new cards to import - database up to date");
            return Ok(Vec::new());
        }
        tracing::info!("importing {} new cards", new_cards.len());
        let cards: Vec<ScryfallCard> = new_cards
            .batch_insert_with_tx(&mut tx, batch_size, sync_metrics)
            .await?;
        tx.commit().await?;
        Ok(cards)
    }
    async fn delete_if_exists_and_batch_insert(
        &self,
        cards: Vec<ScryfallCard>,
        batch_size: usize,
        sync_metrics: &mut SyncMetrics,
    ) -> Result<Vec<ScryfallCard>, CreateCardError> {
        tracing::info!("initiating delete if exists and insert process");
        tracing::info!("received {} cards", cards.len());
        let mut tx = self.pool.begin().await?;
        // extract ids for deletion
        let card_ids: Vec<Uuid> = cards.iter().map(|c| c.id).collect();
        tracing::info!("deleting {} cards", card_ids.len());
        // delete the cards (card_profile cascade cascades)
        query("DELETE FROM scryfall_cards WHERE id = ANY($1)")
            .bind(card_ids)
            .execute(&mut *tx)
            .await?;
        tracing::info!("importing {} cards", cards.len());
        let cards: Vec<ScryfallCard> = cards
            .batch_insert_with_tx(&mut tx, batch_size, sync_metrics)
            .await?;
        tx.commit().await?;
        Ok(cards)
    }
    async fn get_card(&self, request: &GetCardRequest) -> Result<ScryfallCard, GetCardError> {
        let scryfall_card: ScryfallCard = query_as("SELECT * FROM scryfall_cards WHERE id = $1")
            .bind(request.id())
            .fetch_one(&self.pool)
            .await?;

        Ok(scryfall_card)
    }
    async fn get_cards(
        &self,
        request: &GetCardsRequest,
    ) -> Result<Vec<ScryfallCard>, GetCardError> {
        let cards: Vec<ScryfallCard> = query_as("SELECT * FROM scryfall_cards WHERE id = ANY($1)")
            .bind(request.ids())
            .fetch_all(&self.pool)
            .await?;
        Ok(cards)
    }
    async fn search_cards(
        &self,
        request: &SearchCardRequest,
    ) -> Result<Vec<ScryfallCard>, SearchCardError> {
        let mut qb: QueryBuilder<'_, Postgres> = QueryBuilder::new("SELECT * FROM scryfall_cards");

        // early return with all cards if no filtering is applied
        if !request.has_filters() {
            let cards: Vec<ScryfallCard> = qb.build_query_as().fetch_all(&self.pool).await?;
            return Ok(cards);
        }

        // otherwise we start looking through the filters
        let mut sep: Separated<Postgres, &'static str> = qb.separated(" AND ");

        let mut param_index = 1;

        if let Some(name) = &request.name {
            sep.push(format!("name ILIKE ${}", param_index));
            sep.push_bind_unseparated(format!("%{}%", name));
            param_index += 1;
        }
        if let Some(type_line) = &request.type_line {
            sep.push(format!("type_line ILIKE ${}", param_index));
            sep.push_bind_unseparated(format!("%{}%", type_line));
            param_index += 1;
        }
        if let Some(set) = &request.set {
            sep.push(format!("set ILIKE ${}", param_index));
            sep.push_bind_unseparated(format!("%{}%", set));
            param_index += 1;
        }
        if let Some(rarity) = &request.rarity {
            sep.push(format!("rarity ILIKE ${}", param_index));
            sep.push_bind_unseparated(format!("%{}%", rarity));
            param_index += 1;
        }
        if let Some(cmc) = request.cmc {
            sep.push(format!("cmc = ${}", param_index));
            sep.push_bind_unseparated(cmc);
            param_index += 1;
        }
        if let Some(color_identity) = &request.color_identity {
            sep.push(format!("color_identity && ${}", param_index));
            sep.push_bind_unseparated(color_identity);
            param_index += 1;
        }
        if let Some(oracle_text) = &request.oracle_text {
            sep.push(format!("oracle_text ILIKE ${}", param_index));
            sep.push_bind_unseparated(format!("%{}%", oracle_text));
            param_index += 1;
        }
        if let Some(limit) = request.limit {
            qb.push(format!(" LIMIT ${}", param_index));
            qb.push_bind(limit as i32);
            param_index += 1;
        }
        if let Some(offset) = request.offset {
            qb.push(format!(" OFFSET ${}", param_index));
            qb.push_bind(offset as i32);
        }

        let cards: Vec<ScryfallCard> = qb.build_query_as().fetch_all(&self.pool).await?;

        Ok(cards)
    }
    async fn get_card_profile(
        &self,
        request: &GetCardProfileRequest,
    ) -> Result<CardProfile, GetCardProfileError> {
        let database_card_profile = query_as!(
            DatabaseCardProfile,
            "SELECT id, scryfall_card_id FROM card_profiles WHERE id = $1",
            request.id()
        )
        .fetch_one(&self.pool)
        .await?;

        let card_profile = database_card_profile.try_into()?;

        Ok(card_profile)
    }
    async fn get_card_profiles(
        &self,
        request: &GetCardProfilesRequest,
    ) -> Result<Vec<CardProfile>, GetCardProfileError> {
        let database_card_profiles = query_as!(
            DatabaseCardProfile,
            "SELECT id, scryfall_card_id FROM card_profiles WHERE id = ANY($1)",
            request.ids()
        )
        .fetch_all(&self.pool)
        .await?;

        let card_profiles = database_card_profiles
            .into_iter()
            .map(|x| x.try_into())
            .collect::<Result<Vec<CardProfile>, ToCardProfileError>>()?;

        Ok(card_profiles)
    }
    async fn delete_all(&self) -> Result<Vec<ScryfallCard>, anyhow::Error> {
        let cards: Vec<ScryfallCard> = query_as("DELETE FROM scryfall_cards RETURNING *;")
            .fetch_all(&self.pool)
            .await?;
        Ok(cards)
    }
    async fn record_sync_metrics(
        &self,
        sync_metrics: SyncMetrics,
    ) -> Result<SyncMetrics, anyhow::Error> {
        let mut tx = self.pool.begin().await?;
        let query_sql = "INSERT INTO scryfall_card_sync_metrics".to_string()
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
        let last_sync_date: Option<NaiveDateTime> = query_scalar(
            "SELECT started_at FROM scryfall_card_sync_metrics WHERE sync_type = $1 ORDER BY started_at DESC LIMIT 1",
        )
        .bind(sync_type.to_string())
        .fetch_optional(&self.pool)
        .await
        .context("failed to get last sync date")?;

        Ok(last_sync_date)
    }
}
