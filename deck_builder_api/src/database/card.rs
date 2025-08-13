use std::collections::HashSet;

use crate::models::scryfall_card::ScryfallCard;
use itertools::Itertools;
use sqlx::{postgres::PgArguments, query, query::Query, PgPool, Postgres};
use uuid::Uuid;

#[allow(dead_code)]
pub trait Insert {
    async fn insert(self, pg_pool: &PgPool) -> Result<(), sqlx::Error>;
}

impl Insert for ScryfallCard {
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

pub async fn delete_all(pg_pool: &PgPool) -> Result<(), sqlx::Error> {
    query!("DELETE FROM scryfall_cards;")
        .execute(pg_pool)
        .await?;
    Ok(())
}

pub trait BulkInsert {
    async fn bulk_insert(self, pg_pool: &PgPool) -> Result<(), sqlx::Error>;
}

impl BulkInsert for Vec<ScryfallCard> {
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
        sc_query_sql.push_str(" ON CONFLICT (id) DO NOTHING");

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
        cp_query_sql.push_str(" ON CONFLICT (scryfall_card_id) DO NOTHING");

        // build query with all binds
        let mut cp_query = query(cp_query_sql.as_str());
        for id in card_ids {
            cp_query = cp_query.bind_card_profile_fields(id);
        }
        cp_query.execute(pg_pool).await?;

        Ok(())
    }
}

// add these fields later
// will have to build structs for them
// make sure their order matches what is in mod.rs as you add
// all_parts,
// card_faces,
// legalities
// image_uris,
// prices: Prices,
// purchase_uris,
// related_uris,
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
    cmc,
    color_identity,
    color_indicator,
    colors,
    defense,
    edhrec_rank,
    game_changer,
    hand_modifier,
    keywords,
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
    oversized,
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
            // scryfall_card.all_parts
            // scryfall_card.card_faces
            .bind(card.cmc)
            .bind(card.color_identity)
            .bind(card.color_indicator)
            .bind(card.colors)
            .bind(card.defense)
            .bind(card.edhrec_rank)
            .bind(card.game_changer)
            .bind(card.hand_modifier)
            .bind(card.keywords)
            // scryfall_card.legalities
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
            // scryfall_card.image_uris
            .bind(card.oversized)
            // scryfall_card.prices: Prices,
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
