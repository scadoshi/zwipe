// pub mod card_face;
pub mod color;
// pub mod image_uris;
// pub mod legalities;
// pub mod prices;
// pub mod related_card;

use chrono::{NaiveDate, NaiveDateTime};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// use crate::models::card::card_face::CardFace;
use crate::models::card::color::{Color, Colors};
// use crate::models::card::image_uris::ImageUris;
// use crate::models::card::legalities::Legalities;
// use crate::models::card::prices::Prices;
// use crate::models::card::related_card::RelatedCard;
use crate::schema::cards;

/// Complete card data as stored in the database
#[derive(Debug, Clone, Queryable, Selectable, Serialize)]
#[diesel(table_name = cards)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Card {
    // My Fields
    // Those that exist only in my database
    pub id: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,

    // Core Card Fields
    // Cards have the following core properties
    pub arena_id: Option<i32>,
    pub scryfall_id: Uuid,
    pub lang: String,
    pub mtgo_id: Option<i32>,
    pub mtgo_foil_id: Option<i32>,
    pub multiverse_ids: Option<Vec<Option<i32>>>,
    pub tcgplayer_id: Option<i32>,
    pub tcgplayer_etched_id: Option<i32>,
    pub cardmarket_id: Option<i32>,
    pub object: String,
    pub layout: String,
    pub oracle_id: Option<Uuid>,
    pub prints_search_uri: String,
    pub rulings_uri: String,
    pub scryfall_uri: String,
    pub scryfall_api_uri: String,

    // Gameplay Fields
    // Cards have the following properties relevant to the game rules
    // pub all_parts: Option<Vec<RelatedCard>>,
    // pub card_faces: Option<Vec<CardFace>>,
    pub cmc: f64,
    pub color_identity: Option<Vec<Option<String>>>,
    pub color_indicator: Option<Vec<Option<String>>>,
    pub colors: Option<Vec<Option<String>>>,
    pub defense: Option<String>,
    pub edhrec_rank: Option<i32>,
    pub game_changer: Option<bool>,
    pub hand_modifier: Option<String>,
    pub keywords: Option<Vec<Option<String>>>,
    // pub legalities: Legalities,
    pub life_modifier: Option<String>,
    pub loyalty: Option<String>,
    pub mana_cost: Option<String>,
    pub name: String,
    pub oracle_text: Option<String>,
    pub penny_rank: Option<i32>,
    pub power: Option<String>,
    pub produced_mana: Vec<Option<Color>>,
    pub reserved: bool,
    pub toughness: Option<String>,
    pub type_line: String,

    // Print Fields
    // Cards have the following properties unique to their particular re/print
    pub artist: Option<String>,
    pub artist_ids: Option<Vec<Option<Uuid>>>,
    pub attraction_lights: Option<Vec<Option<String>>>,
    pub booster: bool,
    pub border_color: String,
    pub card_back_id: Uuid,
    pub collector_number: String,
    pub content_warning: Option<bool>,
    pub digital: bool,
    pub finishes: Vec<Option<String>>,
    pub flavor_name: Option<String>,
    pub flavor_text: Option<String>,
    pub frame_effects: Option<Vec<Option<String>>>,
    pub frame: String,
    pub full_art: bool,
    pub games: Option<Vec<Option<String>>>,
    pub highres_image: bool,
    pub illustration_id: Option<Uuid>,
    pub image_status: String,
    // pub image_uris: ImageUris,
    pub oversized: bool,
    // pub prices: Prices,
    pub printed_name: Option<String>,
    pub printed_text: Option<String>,
    pub printed_type_line: Option<String>,
    pub promo: bool,
    pub promo_types: Option<Vec<Option<String>>>,
    // pub purchase_uris: Option<serde_json::Value>, // fix later
    pub rarity: String,
    // pub related_uris: serde_json::Value, // fix later
    pub released_at: NaiveDate,
    pub reprint: bool,
    pub scryfall_set_uri: String,
    pub set_name: String,
    pub set_search_uri: String,
    pub set_type: String,
    pub set_uri: String,
    pub set: String,
    pub set_id: Uuid,
    pub story_spotlight: bool,
    pub textless: bool,
    pub variation: bool,
    pub variation_of: Option<Uuid>,
    pub security_stamp: Option<String>,
    pub watermark: Option<String>,
    pub preview_previewed_at: Option<NaiveDate>,
    pub preview_source_uri: Option<String>,
    pub preview_source: Option<String>,
}

/// Data required to create a new card in the database
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = cards)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewCard {
    // Core Card Fields
    // Cards have the following core properties
    pub arena_id: Option<i32>,
    pub scryfall_id: Uuid,
    pub lang: String,
    pub mtgo_id: Option<i32>,
    pub mtgo_foil_id: Option<i32>,
    pub multiverse_ids: Option<Vec<Option<i32>>>,
    pub tcgplayer_id: Option<i32>,
    pub tcgplayer_etched_id: Option<i32>,
    pub cardmarket_id: Option<i32>,
    pub object: String,
    pub layout: String,
    pub oracle_id: Option<Uuid>,
    pub prints_search_uri: String,
    pub rulings_uri: String,
    pub scryfall_uri: String,
    pub scryfall_api_uri: String,

    // Gameplay Fields
    // Cards have the following properties relevant to the game rules
    // pub all_parts: Option<Vec<RelatedCard>>,
    // pub card_faces: Option<Vec<CardFace>>,
    pub cmc: f64,
    pub color_identity: Option<Colors>,
    pub color_indicator: Option<Colors>,
    pub colors: Option<Colors>,
    pub defense: Option<String>,
    pub edhrec_rank: Option<i32>,
    pub game_changer: Option<bool>,
    pub hand_modifier: Option<String>,
    pub keywords: Option<Vec<Option<String>>>,
    // pub legalities: Legalities,
    pub life_modifier: Option<String>,
    pub loyalty: Option<String>,
    pub mana_cost: Option<String>,
    pub name: String,
    pub oracle_text: Option<String>,
    pub penny_rank: Option<i32>,
    pub power: Option<String>,
    pub produced_mana: Vec<Color>,
    pub reserved: bool,
    pub toughness: Option<String>,
    pub type_line: String,

    // Print Fields
    // Cards have the following properties unique to their particular re/print
    pub artist: Option<String>,
    pub artist_ids: Option<Vec<Uuid>>,
    pub attraction_lights: Option<Vec<Option<String>>>,
    pub booster: bool,
    pub border_color: String,
    pub card_back_id: Uuid,
    pub collector_number: String,
    pub content_warning: Option<bool>,
    pub digital: bool,
    pub finishes: Vec<Option<String>>,
    pub flavor_name: Option<String>,
    pub flavor_text: Option<String>,
    pub frame_effects: Option<Vec<Option<String>>>,
    pub frame: String,
    pub full_art: bool,
    pub games: Option<Vec<Option<String>>>,
    pub highres_image: bool,
    pub illustration_id: Option<Uuid>,
    pub image_status: String,
    // pub image_uris: ImageUris,
    pub oversized: bool,
    // pub prices: Prices,
    pub printed_name: Option<String>,
    pub printed_text: Option<String>,
    pub printed_type_line: Option<String>,
    pub promo: bool,
    pub promo_types: Option<Vec<Option<String>>>,
    // pub purchase_uris: Option<serde_json::Value>, // fix later
    pub rarity: String,
    // pub related_uris: serde_json::Value, // fix later
    pub released_at: NaiveDate,
    pub reprint: bool,
    pub scryfall_set_uri: String,
    pub set_name: String,
    pub set_search_uri: String,
    pub set_type: String,
    pub set_uri: String,
    pub set: String,
    pub set_id: Uuid,
    pub story_spotlight: bool,
    pub textless: bool,
    pub variation: bool,
    pub variation_of: Option<Uuid>,
    pub security_stamp: Option<String>,
    pub watermark: Option<String>,
    pub preview_previewed_at: Option<NaiveDate>,
    pub preview_source_uri: Option<String>,
    pub preview_source: Option<String>,
}

/// Partial card data for updating existing cards
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = cards)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UpdateCard {
    // Core Card Fields
    // Cards have the following core properties
    pub arena_id: Option<i32>,
    pub scryfall_id: Option<Uuid>,
    pub lang: Option<String>,
    pub mtgo_id: Option<i32>,
    pub mtgo_foil_id: Option<i32>,
    pub multiverse_ids: Option<Vec<Option<i32>>>,
    pub tcgplayer_id: Option<i32>,
    pub tcgplayer_etched_id: Option<i32>,
    pub cardmarket_id: Option<i32>,
    pub object: Option<String>,
    pub layout: Option<String>,
    pub oracle_id: Option<Uuid>,
    pub prints_search_uri: Option<String>,
    pub rulings_uri: Option<String>,
    pub scryfall_uri: Option<String>,
    pub scryfall_api_uri: Option<String>,

    // Gameplay Fields
    // Cards have the following properties relevant to the game rules
    // pub all_parts: Option<Vec<RelatedCard>>,
    // pub card_faces: Option<Vec<CardFace>>,
    pub cmc: Option<f64>,
    pub color_identity: Option<Vec<Option<String>>>,
    pub color_indicator: Option<Vec<Option<String>>>,
    pub colors: Option<Vec<Option<String>>>,
    pub defense: Option<String>,
    pub edhrec_rank: Option<i32>,
    pub game_changer: Option<bool>,
    pub hand_modifier: Option<String>,
    pub keywords: Option<Vec<Option<String>>>,
    // pub legalities: Option<Legalities>,
    pub life_modifier: Option<String>,
    pub loyalty: Option<String>,
    pub mana_cost: Option<String>,
    pub name: Option<String>,
    pub oracle_text: Option<String>,
    pub penny_rank: Option<i32>,
    pub power: Option<String>,
    pub produced_mana: Option<Vec<Color>>,
    pub reserved: Option<bool>,
    pub toughness: Option<String>,
    pub type_line: Option<String>,

    // Print Fields
    // Cards have the following properties unique to their particular re/print
    pub artist: Option<String>,
    pub artist_ids: Option<Vec<Uuid>>,
    pub attraction_lights: Option<Vec<Option<String>>>,
    pub booster: Option<bool>,
    pub border_color: Option<String>,
    pub card_back_id: Option<Uuid>,
    pub collector_number: Option<String>,
    pub content_warning: Option<bool>,
    pub digital: Option<bool>,
    pub finishes: Option<Vec<Option<String>>>,
    pub flavor_name: Option<String>,
    pub flavor_text: Option<String>,
    pub frame_effects: Option<Vec<Option<String>>>,
    pub frame: Option<String>,
    pub full_art: Option<bool>,
    pub games: Option<Vec<Option<String>>>,
    pub highres_image: Option<bool>,
    pub illustration_id: Option<Uuid>,
    pub image_status: Option<String>,
    // pub image_uris: Option<ImageUris>,
    pub oversized: Option<bool>,
    // pub prices: Option<Prices>,
    pub printed_name: Option<String>,
    pub printed_text: Option<String>,
    pub printed_type_line: Option<String>,
    pub promo: Option<bool>,
    pub promo_types: Option<Vec<Option<String>>>,
    // pub purchase_uris: Option<serde_json::Value>, // fix later
    pub rarity: Option<String>,
    // pub related_uris: Option<serde_json::Value>, // fix later
    pub released_at: Option<NaiveDate>,
    pub reprint: Option<bool>,
    pub scryfall_set_uri: Option<String>,
    pub set_name: Option<String>,
    pub set_search_uri: Option<String>,
    pub set_type: Option<String>,
    pub set_uri: Option<String>,
    pub set: Option<String>,
    pub set_id: Option<Uuid>,
    pub story_spotlight: Option<bool>,
    pub textless: Option<bool>,
    pub variation: Option<bool>,
    pub variation_of: Option<Uuid>,
    pub security_stamp: Option<String>,
    pub watermark: Option<String>,
    pub preview_previewed_at: Option<NaiveDate>,
    pub preview_source_uri: Option<String>,
    pub preview_source: Option<String>,
}

/// Sanitized card data for API responses
#[derive(Debug, Serialize)]
pub struct CardResponse {
    // My Fields
    // Those that exist only in my database
    pub id: i32,
    // pub created_at: NaiveDateTime,
    // pub updated_at: NaiveDateTime,

    // Core Card Fields
    // Cards have the following core properties
    pub arena_id: Option<i32>,
    pub scryfall_id: Uuid,
    pub lang: String,
    pub mtgo_id: Option<i32>,
    pub mtgo_foil_id: Option<i32>,
    pub multiverse_ids: Option<Vec<Option<i32>>>,
    pub tcgplayer_id: Option<i32>,
    pub tcgplayer_etched_id: Option<i32>,
    pub cardmarket_id: Option<i32>,
    pub object: String,
    pub layout: String,
    pub oracle_id: Option<Uuid>,
    pub prints_search_uri: String,
    pub rulings_uri: String,
    pub scryfall_uri: String,
    pub scryfall_api_uri: String,

    // Gameplay Fields
    // Cards have the following properties relevant to the game rules
    // pub all_parts: Option<Vec<RelatedCard>>,
    // pub card_faces: Option<Vec<CardFace>>,
    pub cmc: f64,
    pub color_identity: Option<Vec<Option<String>>>,
    pub color_indicator: Option<Vec<Option<String>>>,
    pub colors: Option<Vec<Option<String>>>,
    pub defense: Option<String>,
    pub edhrec_rank: Option<i32>,
    pub game_changer: Option<bool>,
    pub hand_modifier: Option<String>,
    pub keywords: Option<Vec<Option<String>>>,
    // pub legalities: Legalities,
    pub life_modifier: Option<String>,
    pub loyalty: Option<String>,
    pub mana_cost: Option<String>,
    pub name: String,
    pub oracle_text: Option<String>,
    pub penny_rank: Option<i32>,
    pub power: Option<String>,
    pub produced_mana: Vec<Color>,
    pub reserved: bool,
    pub toughness: Option<String>,
    pub type_line: String,

    // Print Fields
    // Cards have the following properties unique to their particular re/print
    pub artist: Option<String>,
    pub artist_ids: Option<Vec<Uuid>>,
    pub attraction_lights: Option<Vec<Option<String>>>,
    pub booster: bool,
    pub border_color: String,
    pub card_back_id: Uuid,
    pub collector_number: String,
    pub content_warning: Option<bool>,
    pub digital: bool,
    pub finishes: Vec<Option<String>>,
    pub flavor_name: Option<String>,
    pub flavor_text: Option<String>,
    pub frame_effects: Option<Vec<Option<String>>>,
    pub frame: String,
    pub full_art: bool,
    pub games: Option<Vec<Option<String>>>,
    pub highres_image: bool,
    pub illustration_id: Option<Uuid>,
    pub image_status: String,
    // pub image_uris: ImageUris,
    pub oversized: bool,
    // pub prices: Prices,
    pub printed_name: Option<String>,
    pub printed_text: Option<String>,
    pub printed_type_line: Option<String>,
    pub promo: bool,
    pub promo_types: Option<Vec<Option<String>>>,
    // pub purchase_uris: Option<serde_json::Value>, // fix later
    pub rarity: String,
    // pub related_uris: serde_json::Value, // fix later
    pub released_at: NaiveDate,
    pub reprint: bool,
    pub scryfall_set_uri: String,
    pub set_name: String,
    pub set_search_uri: String,
    pub set_type: String,
    pub set_uri: String,
    pub set: String,
    pub set_id: Uuid,
    pub story_spotlight: bool,
    pub textless: bool,
    pub variation: bool,
    pub variation_of: Option<Uuid>,
    pub security_stamp: Option<String>,
    pub watermark: Option<String>,
    pub preview_previewed_at: Option<NaiveDate>,
    pub preview_source_uri: Option<String>,
    pub preview_source: Option<String>,
}
