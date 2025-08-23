// getting the helper implementations
pub mod scryfall_card;
use crate::outbound::sqlx::card::scryfall_card::{
    all_parts, card_faces, image_uris, legalities, prices,
};
use sqlx::{Decode, Encode, Type};

// other internal
use crate::{
    domain::card::{
        models::{scryfall_card::ScryfallCard, CreateCardError, GetCardError},
        ports::CardRepository,
    },
    outbound::sqlx::postgres::Postgres as MyPostgres,
};

// std
use std::{collections::HashSet, future::Future};

// other external
use itertools::Itertools;
use sqlx::{postgres::PgArguments, query, query_as, query_scalar, PgTransaction, Postgres};
use uuid::Uuid;

// ===============================
//              helpers
// ===============================
//
// allows redundant operations within CardRepository
// without having to create new transactions
// these should **not** commit the transaction
// that is the responsibility of higher level functions
//

/// basic insertions like a single card
/// or multiple cards with no special batching
pub trait InsertWithTransaction {
    fn insert_with_tx(
        self,
        tx: &mut PgTransaction<'_>,
    ) -> impl Future<Output = Result<(), CreateCardError>> + Send;
}

/// allows inserting a single card with a given transaction
impl InsertWithTransaction for ScryfallCard {
    async fn insert_with_tx(self, tx: &mut PgTransaction<'_>) -> Result<(), CreateCardError> {
        let scryfall_card_id = self.id.clone();

        let query_sql = format!(
            "INSERT INTO scryfall_cards ({}) VALUES ({})",
            SCRYFALL_CARD_FIELDS,
            (1..=scryfall_card_fieldcount())
                .map(|x| format!("${}", x))
                .join(",")
        );

        query(query_sql.as_str())
            .bind_scryfall_card_fields(self)
            .execute(&mut **tx)
            .await?;

        query("INSERT INTO card_profiles (scryfall_card_id) VALUES ($1)")
            .bind(scryfall_card_id)
            .execute(&mut **tx)
            .await?;

        Ok(())
    }
}

/// allows inserting multiple cards with a single transaction
impl InsertWithTransaction for Vec<ScryfallCard> {
    async fn insert_with_tx(self, tx: &mut PgTransaction<'_>) -> Result<(), CreateCardError> {
        // for building out value tuples
        let card_count = self.len();
        let scryfallcard_field_count = scryfall_card_fieldcount();

        // for inserting into card_profile later
        // HashSet<T> avoids trying dupes
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
        // scryfall_card_query_sql.push_str(" ON CONFLICT (id) DO NOTHING");

        // build query with all binds
        let mut scryfall_card_query = query(scryfall_card_query_sql.as_str());
        for card in self {
            scryfall_card_query = scryfall_card_query.bind_scryfall_card_fields(card);
        }

        scryfall_card_query.execute(&mut **tx).await?;

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
        // card_profile_query_sql.push_str(" ON CONFLICT (scryfall_card_id) DO NOTHING");

        // build query with all binds
        let mut card_profile_query = query(card_profile_query_sql.as_str());
        for id in card_ids {
            card_profile_query = card_profile_query.bind_card_profile_fields(id);
        }
        card_profile_query.execute(&mut **tx).await?;

        Ok(())
    }
}

///
pub trait BatchInsertWithTransaction {
    fn batch_insert_with_tx(
        self,
        tx: &mut PgTransaction<'_>,
        batch_size: usize,
    ) -> impl Future<Output = Result<(), CreateCardError>> + Send;
}

impl BatchInsertWithTransaction for Vec<ScryfallCard> {
    async fn batch_insert_with_tx(
        self,
        tx: &mut PgTransaction<'_>,
        batch_size: usize,
    ) -> Result<(), CreateCardError> {
        let mut failed_batches = 0;
        let mut total_batches = 0;

        for chunk in self.chunks(batch_size) {
            total_batches += 1;
            let owned_chunk: Vec<ScryfallCard> = chunk.to_owned();

            match owned_chunk.insert_with_tx(tx).await {
                Ok(_) => (),
                Err(e) => {
                    failed_batches += 1;
                    tracing::warn!("Batch #{} failed with error: {:?}", total_batches, e);
                    tracing::warn!("Retrying batch #{:?} one card at a time", total_batches);

                    let mut total_card_inserts = 1;
                    let mut failed_card_inserts = 0;
                    for card in chunk.to_owned() {
                        total_card_inserts += 1;

                        let card_name = card.name.clone();
                        let card_id = card.id.clone();

                        match card.insert_with_tx(tx).await {
                            Ok(_) => (),
                            Err(e) => {
                                tracing::warn!(
                                    "Card {:?} ({}) in batch #{} failed with error: {:?}",
                                    card_name,
                                    card_id,
                                    total_batches,
                                    e
                                );
                                failed_card_inserts += 1;
                            }
                        }
                    }
                    tracing::info!(
                        "Batch #{} completed {}/{} inserts",
                        total_batches,
                        total_card_inserts - failed_card_inserts,
                        total_card_inserts
                    );
                }
            }
        }

        tracing::info!(
            "Completed {}/{} batches successfully",
            total_batches - failed_batches,
            total_batches
        );

        Ok(())
    }
}

// ============================
//        main
// ============================
// transaction commits should be handled at this level!

impl CardRepository for MyPostgres {
    // ============================
    //         create
    // ============================
    async fn insert(&self, card: ScryfallCard) -> Result<(), CreateCardError> {
        let mut tx = self.pool.begin().await?;
        card.insert_with_tx(&mut tx).await?;
        tx.commit().await?;
        Ok(())
    }

    async fn bulk_insert(&self, cards: Vec<ScryfallCard>) -> Result<(), CreateCardError> {
        let mut tx = self.pool.begin().await?;
        cards.insert_with_tx(&mut tx).await?;
        tx.commit().await?;
        Ok(())
    }

    async fn batch_insert(
        &self,
        cards: Vec<ScryfallCard>,
        batch_size: usize,
    ) -> Result<(), CreateCardError> {
        let mut tx = self.pool.begin().await?;
        cards.batch_insert_with_tx(&mut tx, batch_size).await?;
        tx.commit().await?;
        Ok(())
    }

    async fn batch_insert_if_not_exists(
        &self,
        cards: Vec<ScryfallCard>,
        batch_size: usize,
    ) -> Result<(), CreateCardError> {
        let mut tx = self.pool.begin().await?;

        let existing_ids: Vec<Uuid> = query_scalar("SELECT id FROM scryfall_cards")
            .fetch_all(&self.pool)
            .await?;

        let new_cards: Vec<ScryfallCard> = cards
            .into_iter()
            .filter(|x| !existing_ids.contains(&x.id))
            .collect();

        tracing::info!("Calculated difference");

        if new_cards.is_empty() {
            tracing::info!("Database up to date");
            return Ok(());
        }

        new_cards.batch_insert_with_tx(&mut tx, batch_size).await?;

        tx.commit().await?;

        Ok(())
    }

    async fn delete_if_exists_and_batch_insert(
        &self,
        cards: Vec<ScryfallCard>,
        batch_size: usize,
    ) -> Result<(), CreateCardError> {
        let mut tx = self.pool.begin().await?;

        // extract ids for deletion
        let card_ids: Vec<Uuid> = cards.iter().map(|c| c.id).collect();

        // delete the cards (card_profile cascade cascades)
        query("DELETE FROM scryfall_cards WHERE id = ANY($1)")
            .bind(card_ids)
            .execute(&mut *tx)
            .await?;

        cards.batch_insert_with_tx(&mut tx, batch_size).await?;

        tx.commit().await?;
        Ok(())
    }

    // ============================
    //         get
    // ============================
    async fn get_card(&self, id: &Uuid) -> Result<ScryfallCard, GetCardError> {
        let card: ScryfallCard = query_as("SELECT * FROM scryfall_cards WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        Ok(card)
    }

    async fn search_cards(
        &self,
        params: crate::domain::card::models::CardSearchParameters,
    ) -> Result<Vec<ScryfallCard>, crate::domain::card::models::CardNotFound> {
        todo!("build search cards function")
    }

    // ============================
    //         delete
    // ============================
    async fn delete_all(&self) -> Result<(), anyhow::Error> {
        //     query("DELETE FROM scryfall_cards;")
        //     .execute(pg_pool)
        //     .await?;
        // Ok(())
        todo!()
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

fn scryfall_card_fieldcount() -> usize {
    SCRYFALL_CARD_FIELDS
        .lines()
        .filter(|x| x.contains(","))
        .count()
        + 1
}

trait BindScryfallCardFields {
    fn bind_scryfall_card_fields(self, card: ScryfallCard) -> Self;
}

impl BindScryfallCardFields for sqlx::query::Query<'_, Postgres, PgArguments> {
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
            .bind(card.attraction_lights)
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
            .bind(card.purchase_uris)
            .bind(card.rarity)
            .bind(card.related_uris)
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
impl BindCardProfileFields for sqlx::query::Query<'_, Postgres, PgArguments> {
    fn bind_card_profile_fields(self, scryfall_card_id: Uuid) -> Self {
        self.bind(scryfall_card_id)
    }
}
