pub mod scryfall_card;
pub mod sync_metrics;

use crate::outbound::sqlx::postgres::Postgres as MyPostgres;
use crate::{
    domain::card::{
        models::{
            scryfall_card::ScryfallCard,
            sync_metrics::{ErrorMetrics, SyncMetrics, SyncType},
            CreateCardError, GetCardError, SearchCardError, SearchCardRequest,
        },
        ports::CardRepository,
    },
    outbound::sqlx::card::sync_metrics::DatabaseSyncMetrics,
};

use anyhow::Context;
use chrono::NaiveDateTime;
use itertools::Itertools;
use sqlx::{query, query_as, query_scalar, PgTransaction, Postgres, Transaction};
use sqlx::{query_builder::Separated, QueryBuilder};
use std::{collections::HashSet, future::Future};
use uuid::Uuid;

/// for binding all of the card fields
/// onto a `Query` or `QueryAs`
macro_rules! bind_scryfall_card_fields {
    ($query:expr, $card:expr) => {
        $query
            // core card fields
            // cards have the following core properties
            .bind($card.arena_id)
            .bind($card.id)
            .bind($card.lang)
            .bind($card.mtgo_id)
            .bind($card.mtgo_foil_id)
            .bind($card.multiverse_ids)
            .bind($card.tcgplayer_id)
            .bind($card.tcgplayer_etched_id)
            .bind($card.cardmarket_id)
            .bind($card.object)
            .bind($card.layout)
            .bind($card.oracle_id)
            .bind($card.prints_search_uri)
            .bind($card.rulings_uri)
            .bind($card.scryfall_uri)
            .bind($card.uri)
            // gameplay fields
            // cards have the following properties relevant to the game rules
            .bind($card.all_parts)
            .bind($card.card_faces)
            .bind($card.cmc)
            .bind($card.color_identity)
            .bind($card.color_indicator)
            .bind($card.colors)
            .bind($card.defense)
            .bind($card.edhrec_rank)
            .bind($card.game_changer)
            .bind($card.hand_modifier)
            .bind($card.keywords)
            .bind($card.legalities)
            .bind($card.life_modifier)
            .bind($card.loyalty)
            .bind($card.mana_cost)
            .bind($card.name)
            .bind($card.oracle_text)
            .bind($card.penny_rank)
            .bind($card.power)
            .bind($card.produced_mana)
            .bind($card.reserved)
            .bind($card.toughness)
            .bind($card.type_line)
            // print fields
            // cards have the following properties unique to their particular re/print
            .bind($card.artist)
            .bind($card.artist_ids)
            .bind($card.attraction_lights)
            .bind($card.booster)
            .bind($card.border_color)
            .bind($card.card_back_id)
            .bind($card.collector_number)
            .bind($card.content_warning)
            .bind($card.digital)
            .bind($card.finishes)
            .bind($card.flavor_name)
            .bind($card.flavor_text)
            .bind($card.frame_effects)
            .bind($card.frame)
            .bind($card.full_art)
            .bind($card.games)
            .bind($card.highres_image)
            .bind($card.illustration_id)
            .bind($card.image_status)
            .bind($card.image_uris)
            .bind($card.oversized)
            .bind($card.prices)
            .bind($card.printed_name)
            .bind($card.printed_text)
            .bind($card.printed_type_line)
            .bind($card.promo)
            .bind($card.promo_types)
            .bind($card.purchase_uris)
            .bind($card.rarity)
            .bind($card.related_uris)
            .bind($card.released_at)
            .bind($card.reprint)
            .bind($card.scryfall_set_uri)
            .bind($card.set_name)
            .bind($card.set_search_uri)
            .bind($card.set_type)
            .bind($card.set_uri)
            .bind($card.set)
            .bind($card.set_id)
            .bind($card.story_spotlight)
            .bind($card.textless)
            .bind($card.variation)
            .bind($card.variation_of)
            .bind($card.security_stamp)
            .bind($card.watermark)
            .bind($card.preview_previewed_at)
            .bind($card.preview_source_uri)
            .bind($card.preview_source)
    };
}

// ===============================
//         helpers (*3*)
// ===============================
//
// allows redundant operations within `CardRepository`
// without having to create new transactions
// these should **not** commit the transaction
// that is the responsibility of higher level functions

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

        let query_sql = format!(
            "INSERT INTO scryfall_cards ({}) VALUES ({}) RETURNING *",
            SCRYFALL_CARD_FIELDS,
            (1..=scryfall_card_field_count())
                .map(|x| format!("${}", x))
                .join(",")
        );

        let card: ScryfallCard = bind_scryfall_card_fields!(query_as(query_sql.as_str()), self)
            .fetch_one(&mut **tx)
            .await?;

        query("INSERT INTO card_profiles (scryfall_card_id) VALUES ($1)")
            .bind(scryfall_card_id)
            .execute(&mut **tx)
            .await?;

        Ok(card)
    }
}

/// for inserting multiple cards given a transaction
impl InsertWithTransaction for Vec<ScryfallCard> {
    async fn insert_with_tx(self, tx: &mut PgTransaction<'_>) -> Result<Self, CreateCardError> {
        // for building out value tuples
        let card_count = self.len();
        let scryfallcard_field_count = scryfall_card_field_count();

        // for inserting into card_profile later
        // hashset avoids duplicates
        let card_ids: HashSet<Uuid> = self.iter().map(|x| x.id.to_owned()).collect();

        // time to insert into scryfall_card!
        // intializing query sql
        let mut scryfall_card_query_sql = format!(
            "INSERT INTO scryfall_cards ({}) VALUES",
            SCRYFALL_CARD_FIELDS
        );

        // build value tuples
        // like ($1,$2,$3,...), ($80,$81,$82,...), ...
        for i in 0..card_count {
            let start = 1 + (scryfallcard_field_count * i);
            let finish = scryfallcard_field_count + (scryfallcard_field_count * i);

            if i > 0 {
                scryfall_card_query_sql.push(',');
            }

            scryfall_card_query_sql.push('(');
            scryfall_card_query_sql.push_str(
                (start..=finish)
                    .map(|x| format!("${}", x))
                    .join(",")
                    .as_str(),
            );
            scryfall_card_query_sql.push(')');
        }
        scryfall_card_query_sql.push_str(" RETURNING *");

        // build query with all binds
        let mut scryfall_card_query = query_as(scryfall_card_query_sql.as_str());
        for card in self {
            scryfall_card_query = bind_scryfall_card_fields!(scryfall_card_query, card);
        }

        // execute
        let cards: Vec<ScryfallCard> = scryfall_card_query.fetch_all(&mut **tx).await?;

        // time to insert into card_profile!
        // intializing query sql
        let mut card_profile_query_sql: String =
            "INSERT INTO card_profiles (scryfall_card_id) VALUES".to_string();

        // build values tuples
        // like ($1), ($2), ...
        // much simpler than above because we only need to insert a single field
        for i in 0..card_count {
            if i > 0 {
                card_profile_query_sql.push(',');
            }

            card_profile_query_sql.push('(');
            card_profile_query_sql.push_str(format!("${}", i + 1).as_str());
            card_profile_query_sql.push(')');
        }

        // build query with all binds
        let mut card_profile_query = query(card_profile_query_sql.as_str());
        for id in card_ids {
            card_profile_query = card_profile_query.bind(id);
        }
        // execute
        card_profile_query.execute(&mut **tx).await?;

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

const POSTGRES_TX_ABORT_MESSAGE: &str = "current transaction is aborted";

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

// tx commits should be handled at this level
// rather than above
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
    async fn get_card(&self, id: &Uuid) -> Result<ScryfallCard, GetCardError> {
        let card: ScryfallCard = query_as("SELECT * FROM scryfall_cards WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        Ok(card)
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

// fields for use in query field tuples
const SCRYFALL_CARD_FIELDS: &str = r#"
    arena_id,
    id,
    lang,
    mtgo_id,
    mtgo_foil_id,
    multiverse_ids,
    tcgplayer_id,
    tcgplayer_etched_id,
    cardmarket_id,
    object,
    layout,
    oracle_id,
    prints_search_uri,
    rulings_uri,
    scryfall_uri,
    uri,
    all_parts,
    card_faces,
    cmc,
    color_identity,
    color_indicator,
    colors,
    defense,
    edhrec_rank,
    game_changer,
    hand_modifier,
    keywords,
    legalities,
    life_modifier,
    loyalty,
    mana_cost,
    name,
    oracle_text,
    penny_rank,
    power,
    produced_mana,
    reserved,
    toughness,
    type_line,
    artist,
    artist_ids,
    attraction_lights,
    booster,
    border_color,
    card_back_id,
    collector_number,
    content_warning,
    digital,
    finishes,
    flavor_name,
    flavor_text,
    frame_effects,
    frame,
    full_art,
    games,
    highres_image,
    illustration_id,
    image_status,
    image_uris,
    oversized,
    prices,
    printed_name,
    printed_text,
    printed_type_line,
    promo,
    promo_types,
    purchase_uris,
    rarity,
    related_uris,
    released_at,
    reprint,
    scryfall_set_uri,
    set_name,
    set_search_uri,
    set_type,
    set_uri,
    set,
    set_id,
    story_spotlight,
    textless,
    variation,
    variation_of,
    security_stamp,
    watermark,
    preview_previewed_at,
    preview_source_uri,
    preview_source
"#;

pub fn scryfall_card_field_count() -> usize {
    SCRYFALL_CARD_FIELDS
        .lines()
        .filter(|x| x.contains(","))
        .count()
        + 1
}
