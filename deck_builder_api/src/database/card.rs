use std::{collections::HashSet, error::Error as StdError};

use crate::{models::scryfall_card::ScryfallCard, scryfall::fetch_oracle_cards};
use itertools::Itertools;
use sqlx::{postgres::PgArguments, query, query::Query, query_scalar, PgPool, Postgres};
use tracing::{info, warn};
use uuid::Uuid;

pub trait SingleInsert {
    async fn insert(self, pg_pool: &PgPool) -> Result<(), sqlx::Error>;
}

impl SingleInsert for ScryfallCard {
    async fn insert(self, pg_pool: &PgPool) -> Result<(), sqlx::Error> {
        let scryfall_card_id = self.id.clone();
        let query_sql = format!(
            "INSERT INTO scryfall_cards ({}) VALUES ({})",
            SCRYFALL_CARD_FIELDS,
            (1..=sc_fieldcount()).map(|x| format!("${}", x)).join(",")
        );
        query(query_sql.as_str())
            .bind_scryfall_card_fields(self)
            .execute(pg_pool)
            .await?;
        query("INSERT INTO card_profiles (scryfall_card_id, created_at, updated_at) VALUES ($1, $2, $3)")
            .bind(scryfall_card_id)
            .bind(chrono::Utc::now().naive_utc())
            .bind(chrono::Utc::now().naive_utc())
            .execute(pg_pool)
            .await?;
        Ok(())
    }
}

#[allow(dead_code)]
pub async fn delete_all(pg_pool: &PgPool) -> Result<(), sqlx::Error> {
    query("DELETE FROM scryfall_cards;")
        .execute(pg_pool)
        .await?;
    Ok(())
}

pub trait MultipleInsert {
    async fn bulk_insert(self, pg_pool: &PgPool) -> Result<(), sqlx::Error>;
    async fn batch_insert(self, batch_size: usize, pg_pool: &PgPool) -> Result<(), sqlx::Error>;
    async fn smart_insert(self, batch_size: usize, pg_pool: &PgPool) -> Result<(), sqlx::Error>;
}

impl MultipleInsert for Vec<ScryfallCard> {
    async fn bulk_insert(self, pg_pool: &PgPool) -> Result<(), sqlx::Error> {
        // for building out value tuples
        let card_count = self.len();
        let sc_fieldcount = sc_fieldcount();

        // for inserting into card_profile later
        // HashSet<T> avoids trying dupes
        let card_ids: HashSet<Uuid> = self.iter().map(|x| x.id.to_owned()).collect();

        // time to insert into scryfall_card!
        // intializing query sql
        let mut sc_query_sql = format!(
            "INSERT INTO scryfall_cards ({}) VALUES",
            SCRYFALL_CARD_FIELDS
        );

        // build value tuples
        // like ($1,$2,$3,...), ($80,$81,$82,...), ...
        for i in 0..card_count {
            let start = 1 + (sc_fieldcount * i);
            let finish = sc_fieldcount + (sc_fieldcount * i);

            if i > 0 {
                sc_query_sql.push(',');
            }

            sc_query_sql.push('(');
            sc_query_sql.push_str(
                (start..=finish)
                    .map(|x| format!("${}", x))
                    .join(",")
                    .as_str(),
            );
            sc_query_sql.push(')');
        }
        // sc_query_sql.push_str(" ON CONFLICT (id) DO NOTHING");

        // build query with all binds
        let mut sc_query = query(sc_query_sql.as_str());
        for card in self {
            sc_query = sc_query.bind_scryfall_card_fields(card);
        }
        sc_query.execute(pg_pool).await?;

        // time to inset into card_profile!
        // intializing query sql
        let mut cp_query_sql: String =
            "INSERT INTO card_profiles (scryfall_card_id, created_at, updated_at) VALUES"
                .to_string();
        let cp_field_count = 3;

        // build values tuples
        // like ($1,$2,$3), ($4,$5,$6), ...
        for i in 0..card_count {
            let start = 1 + (cp_field_count * i);
            let finish = cp_field_count + (cp_field_count * i);

            if i > 0 {
                cp_query_sql.push(',');
            }

            cp_query_sql.push('(');
            cp_query_sql.push_str(
                (start..=finish)
                    .map(|x| format!("${}", x))
                    .join(",")
                    .as_str(),
            );
            cp_query_sql.push(')');
        }
        // cp_query_sql.push_str(" ON CONFLICT (scryfall_card_id) DO NOTHING");

        // build query with all binds
        let mut cp_query = query(cp_query_sql.as_str());
        for id in card_ids {
            cp_query = cp_query.bind_card_profile_fields(id);
        }
        cp_query.execute(pg_pool).await?;

        Ok(())
    }
    async fn batch_insert(self, batch_size: usize, pg_pool: &PgPool) -> Result<(), sqlx::Error> {
        let mut failed_batches = 0;
        let mut total_batches = 0;

        for chunk in self.chunks(batch_size) {
            total_batches += 1;
            let owned_chunk: Vec<ScryfallCard> = chunk.to_owned();

            match owned_chunk.bulk_insert(pg_pool).await {
                Ok(_) => (),
                Err(e) => {
                    failed_batches += 1;
                    warn!("Batch #{} failed with error: {:?}", total_batches, e);
                    warn!("Retrying batch #{:?} one card at a time", total_batches);

                    let mut total_card_inserts = 1;
                    let mut failed_card_inserts = 0;
                    for card in chunk.to_owned() {
                        total_card_inserts += 1;

                        let card_name = card.name.clone();
                        let card_id = card.id.clone();

                        match card.insert(pg_pool).await {
                            Ok(_) => (),
                            Err(e) => {
                                warn!(
                                    "Card {:?} ({}) in batch #{} failed with error: {:?}",
                                    card_name, card_id, total_batches, e
                                );
                                failed_card_inserts += 1;
                            }
                        }
                    }
                    info!(
                        "Batch #{} completed {}/{} inserts",
                        total_batches,
                        total_card_inserts - failed_card_inserts,
                        total_card_inserts
                    );
                }
            }
        }

        info!(
            "Completed {}/{} batches successfully",
            total_batches - failed_batches,
            total_batches
        );
        Ok(())
    }
    async fn smart_insert(self, batch_size: usize, pg_pool: &PgPool) -> Result<(), sqlx::Error> {
        let existing_ids: Vec<Uuid> = query_scalar("SELECT id FROM scryfall_cards")
            .fetch_all(pg_pool)
            .await?;

        let new_cards: Vec<ScryfallCard> = self
            .into_iter()
            .filter(|x| !existing_ids.contains(&x.id))
            .collect();
        info!("Calculated difference");

        if new_cards.is_empty() {
            info!("Database up to date");
            return Ok(());
        }

        new_cards.batch_insert(batch_size, pg_pool).await?;

        Ok(())
    }
}

pub async fn scryfall_sync(pg_pool: &PgPool) -> Result<(), Box<dyn StdError>> {
    delete_all(&pg_pool).await?;
    // info!("Deleted all cards ");
    info!("Beginning Scryfall sync");
    info!("Fetching oracle cards");
    let bulk_data: Vec<ScryfallCard> = fetch_oracle_cards().await?;
    info!("Scryfall returned {} cards", bulk_data.len());
    let scryfall_cards_row_count: i64 = query_scalar("SELECT COUNT(id) FROM scryfall_cards")
        .fetch_one(pg_pool)
        .await?;
    info!("Database has {} cards", scryfall_cards_row_count);
    let batch_size = 500;
    info!("Smart inserting by batch size {:?}", batch_size);
    bulk_data.smart_insert(batch_size, &pg_pool).await?;
    let scryfall_cards_row_count: i64 = query_scalar("SELECT COUNT(id) FROM scryfall_cards")
        .fetch_one(pg_pool)
        .await?;
    info!("Database now has {} cards", scryfall_cards_row_count);

    info!("Database sync completed");
    Ok(())
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
    rarity,
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

fn sc_fieldcount() -> usize {
    SCRYFALL_CARD_FIELDS
        .lines()
        .filter(|x| x.contains(","))
        .count()
        + 1
}

trait BindScryfallCardFields {
    fn bind_scryfall_card_fields(self, card: ScryfallCard) -> Self;
}

impl BindScryfallCardFields for Query<'_, Postgres, PgArguments> {
    fn bind_scryfall_card_fields(self, card: ScryfallCard) -> Self {
        self
            // Core Card Fields
            // Cards have the following core properties
            .bind(card.arena_id)
            .bind(card.id)
            .bind(card.lang)
            .bind(card.mtgo_id)
            .bind(card.mtgo_foil_id)
            .bind(card.multiverse_ids)
            .bind(card.tcgplayer_id)
            .bind(card.tcgplayer_etched_id)
            .bind(card.cardmarket_id)
            .bind(card.object)
            .bind(card.layout)
            .bind(card.oracle_id)
            .bind(card.prints_search_uri)
            .bind(card.rulings_uri)
            .bind(card.scryfall_uri)
            .bind(card.uri)
            // Gameplay Fields
            // Cards have the following properties relevant to the game rules
            .bind(card.all_parts)
            .bind(card.card_faces)
            .bind(card.cmc)
            .bind(card.color_identity)
            .bind(card.color_indicator)
            .bind(card.colors)
            .bind(card.defense)
            .bind(card.edhrec_rank)
            .bind(card.game_changer)
            .bind(card.hand_modifier)
            .bind(card.keywords)
            .bind(card.legalities)
            .bind(card.life_modifier)
            .bind(card.loyalty)
            .bind(card.mana_cost)
            .bind(card.name)
            .bind(card.oracle_text)
            .bind(card.penny_rank)
            .bind(card.power)
            .bind(card.produced_mana)
            .bind(card.reserved)
            .bind(card.toughness)
            .bind(card.type_line)
            // Print Fields
            // Cards have the following properties unique to their particular re/print
            .bind(card.artist)
            .bind(card.artist_ids)
            .bind(card.booster)
            .bind(card.border_color)
            .bind(card.card_back_id)
            .bind(card.collector_number)
            .bind(card.content_warning)
            .bind(card.digital)
            .bind(card.finishes)
            .bind(card.flavor_name)
            .bind(card.flavor_text)
            .bind(card.frame_effects)
            .bind(card.frame)
            .bind(card.full_art)
            .bind(card.games)
            .bind(card.highres_image)
            .bind(card.illustration_id)
            .bind(card.image_status)
            .bind(card.image_uris)
            .bind(card.oversized)
            .bind(card.prices)
            .bind(card.printed_name)
            .bind(card.printed_text)
            .bind(card.printed_type_line)
            .bind(card.promo)
            .bind(card.promo_types)
            // scryfall_card.purchase_uris // fix later
            .bind(card.rarity)
            // scryfall_card.related_uris // fix later
            .bind(card.released_at)
            .bind(card.reprint)
            .bind(card.scryfall_set_uri)
            .bind(card.set_name)
            .bind(card.set_search_uri)
            .bind(card.set_type)
            .bind(card.set_uri)
            .bind(card.set)
            .bind(card.set_id)
            .bind(card.story_spotlight)
            .bind(card.textless)
            .bind(card.variation)
            .bind(card.variation_of)
            .bind(card.security_stamp)
            .bind(card.watermark)
            .bind(card.preview_previewed_at)
            .bind(card.preview_source_uri)
            .bind(card.preview_source)
    }
}

trait BindCardProfileFields {
    fn bind_card_profile_fields(self, scryfall_card_uuid: Uuid) -> Self;
}
impl BindCardProfileFields for Query<'_, Postgres, PgArguments> {
    fn bind_card_profile_fields(self, scryfall_card_id: Uuid) -> Self {
        self.bind(scryfall_card_id)
            .bind(chrono::Utc::now().naive_utc())
            .bind(chrono::Utc::now().naive_utc())
    }
}
