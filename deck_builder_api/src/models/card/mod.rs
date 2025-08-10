use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::card::colors::Color;

// pub mod card_face;
pub mod colors;
// pub mod image_uris;
// pub mod legalities;
// pub mod prices;
// pub mod related_card;

// - [ ] Color
// - [ ] ImageUris
// - [ ] Legalities
// - [ ] Prices
// - [ ] RelatedCard

/// Card profile data linked to ScryfallCard
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Card {
    pub id: Option<i32>,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
    pub scryfall_card_id: i32,
}

/// Card data from scryfall
#[derive(Debug, Clone, Deserialize, Serialize, sqlx::FromRow)]
pub struct ScryfallCard {
    // Core Card Fields
    // Cards have the following core properties
    pub arena_id: Option<i32>,
    pub id: uuid::Uuid,
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
    pub uri: String,

    // Gameplay Fields
    // Cards have the following properties relevant to the game rules
    // pub all_parts: Option<Vec<RelatedCard>>,
    // pub card_faces: Option<Vec<CardFace>>,
    pub cmc: f64,
    pub color_identity: Option<Vec<Option<Color>>>,
    pub color_indicator: Option<Vec<Option<Color>>>,
    pub colors: Option<Vec<Option<Color>>>,
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
    pub produced_mana: Option<Vec<Option<Color>>>,
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
    pub card_back_id: uuid::Uuid,
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
    pub variation_of: Option<Uuid>,
    pub security_stamp: Option<String>,
    pub watermark: Option<String>,
    pub preview_previewed_at: Option<chrono::NaiveDate>,
    pub preview_source_uri: Option<String>,
    pub preview_source: Option<String>,
}
