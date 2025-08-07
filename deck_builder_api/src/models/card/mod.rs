use chrono::{NaiveDate, NaiveDateTime};
use diesel::{prelude::*, sql_types::Uuid};
use serde::{Deserialize, Serialize};

use crate::{models::card::related_card::RelatedCard, schema::cards};

pub mod card_face;
pub mod colors;
pub mod image_uris;
pub mod legalities;
pub mod prices;
pub mod purchase_uris;
pub mod related_card;
use card_face::CardFace;
use colors::Color;
use image_uris::ImageUris;
use legalities::Legalities;
use prices::Prices;
use purchase_uris::PurchaseUris;
use related_card::RelatedCard;

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
    pub multiverse_ids: Option<Vec<i32>>,
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
    pub all_parts: Option<Vec<RelatedCard>>,
    pub card_faces: Option<Vec<CardFace>>,
    pub cmc: f32,
    pub color_identity: Option<Vec<String>>,
    pub color_indicator: Option<Vec<String>>,
    pub colors: Option<Vec<String>>,
    pub defense: Option<String>,
    pub edhrec_rank: Option<i32>,
    pub game_changer: Option<bool>,
    pub hand_modifier: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub legalities: Legalities,
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
    pub attraction_lights: Option<Vec<String>>,
    pub booster: bool,
    pub border_color: String,
    pub card_back_id: Uuid,
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
    pub illustration_id: Option<Uuid>,
    pub image_status: String,
    pub image_uris: ImageUris,
    pub oversized: bool,
    pub prices: Prices,
    pub printed_name: Option<String>,
    pub printed_text: Option<String>,
    pub printed_type_line: Option<String>,
    pub promo: bool,
    pub promo_types: Option<Vec<String>>,
    pub purchase_uris: Option<serde_json::Value>, // fix later
    pub rarity: String,
    pub related_uris: serde_json::Value, // fix later
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
    pub name: String,
    pub mana_cost: Option<String>,
    pub type_line: String,
    pub rarity: String,
    pub oracle_text: Option<String>,
}

/// Partial card data for updating existing cards
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = cards)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UpdateCard {
    pub name: Option<String>,
    pub mana_cost: Option<String>,
    pub type_line: Option<String>,
    pub rarity: Option<String>,
    pub oracle_text: Option<String>,
}

/// Sanitized card data for API responses
#[derive(Debug, Serialize)]
pub struct CardResponse {
    pub id: i32,
    pub name: String,
    pub mana_cost: Option<String>,
    pub type_line: String,
    pub rarity: String,
    pub oracle_text: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
