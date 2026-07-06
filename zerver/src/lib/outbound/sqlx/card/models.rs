//! Database-to-domain conversion for Scryfall card data.

use sqlx::types::Json;
use sqlx_macros::FromRow;
use zwipe_core::domain::card::scryfall_data::ScryfallData;
use zwipe_core::domain::card::scryfall_data::all_parts::AllParts;
use zwipe_core::domain::card::scryfall_data::card_faces::CardFaces;
use zwipe_core::domain::card::scryfall_data::colors::Colors;
use zwipe_core::domain::card::scryfall_data::image_uris::ImageUris;
use zwipe_core::domain::card::scryfall_data::legalities::Legalities;
use zwipe_core::domain::card::scryfall_data::prices::Prices;
use zwipe_core::domain::card::scryfall_data::rarity::Rarity;

/// Raw database Scryfall data record (unvalidated data from PostgreSQL).
///
/// Fields that are domain newtypes in [`ScryfallData`] are stored here as
/// primitives or `Json<T>` wrappers that SQLx handles natively.
#[derive(Debug, FromRow)]
#[allow(missing_docs)]
pub struct DatabaseScryfallData {
    // Core Card Fields
    pub arena_id: Option<i32>,
    pub id: uuid::Uuid,
    pub lang: String,
    pub mtgo_id: Option<i32>,
    pub mtgo_foil_id: Option<i32>,
    pub multiverse_ids: Option<Vec<i32>>,
    pub tcgplayer_id: Option<i32>,
    pub tcgplayer_etched_id: Option<i32>,
    pub cardmarket_id: Option<i32>,
    pub object: String,
    pub layout: String,
    pub oracle_id: Option<uuid::Uuid>,
    pub prints_search_uri: String,
    pub rulings_uri: String,
    pub scryfall_uri: String,
    pub uri: String,

    // Gameplay Fields (these 9 differ from domain types)
    pub all_parts: Option<Json<AllParts>>,
    pub card_faces: Option<Json<CardFaces>>,
    pub cmc: Option<f64>,
    pub color_identity: Vec<String>,
    pub color_indicator: Option<Vec<String>>,
    pub colors: Option<Vec<String>>,
    pub defense: Option<String>,
    pub edhrec_rank: Option<i32>,
    pub game_changer: Option<bool>,
    pub hand_modifier: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub legalities: Json<Legalities>,
    pub life_modifier: Option<String>,
    pub loyalty: Option<String>,
    pub mana_cost: Option<String>,
    pub name: String,
    pub oracle_text: Option<String>,
    pub penny_rank: Option<i32>,
    pub power: Option<String>,
    pub produced_mana: Option<Vec<String>>,
    pub reserved: bool,
    pub toughness: Option<String>,
    pub type_line: Option<String>,

    // Print Fields
    pub artist: Option<String>,
    pub artist_ids: Option<Vec<uuid::Uuid>>,
    pub attraction_lights: Option<Vec<String>>,
    pub booster: bool,
    pub border_color: String,
    pub card_back_id: Option<uuid::Uuid>,
    pub collector_number: String,
    pub content_warning: Option<bool>,
    pub digital: bool,
    pub finishes: Vec<String>,
    pub flavor_name: Option<String>,
    pub flavor_text: Option<String>,
    pub frame_effects: Option<Vec<String>>,
    pub frame: String,
    pub full_art: bool,
    pub games: Option<Vec<String>>,
    pub highres_image: bool,
    pub illustration_id: Option<uuid::Uuid>,
    pub image_status: String,
    pub image_uris: Option<Json<ImageUris>>,
    pub oversized: bool,
    pub prices: Json<Prices>,
    pub printed_name: Option<String>,
    pub printed_text: Option<String>,
    pub printed_type_line: Option<String>,
    pub promo: bool,
    pub promo_types: Option<Vec<String>>,
    pub purchase_uris: Option<serde_json::Value>,
    pub rarity: String,
    pub related_uris: serde_json::Value,
    pub released_at: chrono::NaiveDate,
    pub reprint: bool,
    pub scryfall_set_uri: String,
    pub set_name: String,
    pub set_search_uri: String,
    pub set_type: String,
    pub set_uri: String,
    pub set: String,
    pub set_id: uuid::Uuid,
    pub story_spotlight: bool,
    pub textless: bool,
    pub variation: bool,
    pub variation_of: Option<uuid::Uuid>,
    pub security_stamp: Option<String>,
    pub watermark: Option<String>,
    pub preview_previewed_at: Option<chrono::NaiveDate>,
    pub preview_source_uri: Option<String>,
    pub preview_source: Option<String>,
}

impl TryFrom<DatabaseScryfallData> for ScryfallData {
    type Error = anyhow::Error;

    fn try_from(db: DatabaseScryfallData) -> Result<Self, Self::Error> {
        Ok(ScryfallData {
            // Core Card Fields — pass through
            arena_id: db.arena_id,
            id: db.id,
            lang: db.lang,
            mtgo_id: db.mtgo_id,
            mtgo_foil_id: db.mtgo_foil_id,
            multiverse_ids: db.multiverse_ids,
            tcgplayer_id: db.tcgplayer_id,
            tcgplayer_etched_id: db.tcgplayer_etched_id,
            cardmarket_id: db.cardmarket_id,
            object: db.object,
            layout: db.layout,
            oracle_id: db.oracle_id,
            prints_search_uri: db.prints_search_uri,
            rulings_uri: db.rulings_uri,
            scryfall_uri: db.scryfall_uri,
            uri: db.uri,

            // Gameplay Fields — convert special types
            all_parts: db.all_parts.map(|j| j.0),
            card_faces: db.card_faces.map(|j| j.0),
            cmc: db.cmc,
            color_identity: Colors::from_short_names(db.color_identity)?,
            color_indicator: db
                .color_indicator
                .map(Colors::from_short_names)
                .transpose()?,
            colors: db.colors.map(Colors::from_short_names).transpose()?,
            defense: db.defense,
            edhrec_rank: db.edhrec_rank,
            game_changer: db.game_changer,
            hand_modifier: db.hand_modifier,
            keywords: db.keywords,
            legalities: db.legalities.0,
            life_modifier: db.life_modifier,
            loyalty: db.loyalty,
            mana_cost: db.mana_cost,
            name: db.name,
            oracle_text: db.oracle_text,
            penny_rank: db.penny_rank,
            power: db.power,
            produced_mana: db.produced_mana,
            reserved: db.reserved,
            toughness: db.toughness,
            type_line: db.type_line,

            // Print Fields — convert special types, pass through rest
            artist: db.artist,
            artist_ids: db.artist_ids,
            attraction_lights: db.attraction_lights,
            booster: db.booster,
            border_color: db.border_color,
            card_back_id: db.card_back_id,
            collector_number: db.collector_number,
            content_warning: db.content_warning,
            digital: db.digital,
            finishes: db.finishes,
            flavor_name: db.flavor_name,
            flavor_text: db.flavor_text,
            frame_effects: db.frame_effects,
            frame: db.frame,
            full_art: db.full_art,
            games: db.games,
            highres_image: db.highres_image,
            illustration_id: db.illustration_id,
            image_status: db.image_status,
            image_uris: db.image_uris.map(|j| j.0),
            oversized: db.oversized,
            prices: db.prices.0,
            printed_name: db.printed_name,
            printed_text: db.printed_text,
            printed_type_line: db.printed_type_line,
            promo: db.promo,
            promo_types: db.promo_types,
            purchase_uris: db.purchase_uris,
            rarity: Rarity::try_from(db.rarity)?,
            related_uris: db.related_uris,
            released_at: db.released_at,
            reprint: db.reprint,
            scryfall_set_uri: db.scryfall_set_uri,
            set_name: db.set_name,
            set_search_uri: db.set_search_uri,
            set_type: db.set_type,
            set_uri: db.set_uri,
            set: db.set,
            set_id: db.set_id,
            story_spotlight: db.story_spotlight,
            textless: db.textless,
            variation: db.variation,
            variation_of: db.variation_of,
            security_stamp: db.security_stamp,
            watermark: db.watermark,
            preview_previewed_at: db.preview_previewed_at,
            preview_source_uri: db.preview_source_uri,
            preview_source: db.preview_source,
        })
    }
}
